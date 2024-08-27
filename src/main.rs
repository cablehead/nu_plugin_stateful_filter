use nu_plugin::{serve_plugin, EvaluatedCall, JsonSerializer};
use nu_plugin::{EngineInterface, PluginCommand};

use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Signature, SyntaxShape,
    Type, Value,
};

struct Plugin;

impl nu_plugin::Plugin for Plugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Command)]
    }
}

struct Command;

impl PluginCommand for Command {
    type Plugin = Plugin;

    fn name(&self) -> &str {
        "stateful filter"
    }

    fn usage(&self) -> &str {
        "Run closure on each element of a list"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::ListStream, Type::ListStream)
            .required(
                "initial",
                SyntaxShape::Any,
                "The initial state to pass to the closure",
            )
            .required(
                "closure",
                SyntaxShape::Closure(Some(vec![SyntaxShape::Any, SyntaxShape::Any])),
                "The closure receives `|state, value|`, and should return
                a record in the shape: { out?: value, state?: new_state }. Both out and state are
                optional. You can drop rows by ommiting the `out` key, and the current state is
                preserved if its key is omitted.",
            )
    }

    fn run(
        &self,
        _plugin: &Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let engine = engine.clone();

        let mut state: Value = call.req(0)?;
        let closure = call.req(1)?;

        let pipeline = input
            .into_iter()
            // append a nothing value to the end of the stream to give the closure a chance to
            // values once the stream is exhausted
            .chain(std::iter::once(Value::nothing(call.head)))
            .filter_map(move |item| {
                let span = item.span();
                match engine.eval_closure(&closure, vec![state.clone(), item.clone()], None) {
                    Ok(value) => {
                        let record = match value.into_record() {
                            Ok(record) => record,
                            Err(err) => return Some(Value::error(err, span)),
                        };

                        if let Some(value) = record.get("state") {
                            state = value.clone();
                        }

                        record.get("out").cloned()
                    }
                    Err(err) => Some(Value::error(err, span)),
                }
            });

        Ok(pipeline.into_pipeline_data(call.head, Signals::empty()))
    }
}

fn main() {
    serve_plugin(&Plugin, JsonSerializer)
}
