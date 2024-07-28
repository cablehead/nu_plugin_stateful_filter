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
                "The closure to evaluate",
            )
    }

    /*
     let head = call.head;
     let engine = engine.clone();
     let initial: Value = call.req(0)?;
     let closure = call.req(1)?;

     let mut next = (!initial.is_nothing()).then_some(initial);

     Ok(std::iter::from_fn(move || {
        next.take()
            .and_then(|value| {
                engine
                    .eval_closure(&closure, vec![value.clone()], Some(value))
                    .and_then(|record| {
                        if record.is_nothing() {
                            Ok(None)
                        } else {
                            let record = record.as_record()?;
                            next = record.get("next").cloned();
                            Ok(record.get("out").cloned())
                        }
                    })
                    .transpose()
            })
            .map(|result| result.unwrap_or_else(|err| Value::error(err, head)))
     )
     into_pipeline_data(head, Signals::empty()))
    */

    fn run(
        &self,
        _plugin: &Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let engine = engine.clone();

        let initial: Value = call.req(0)?;
        let closure = call.req(1)?;

        let mut state = initial;

        let pipeline = input.into_iter().filter_map(move |item| {
            let span = item.span();
            match engine.eval_closure(&closure, vec![state.clone(), item.clone()], Some(item)) {
                Ok(value) => {
                    let record = match value.into_record() {
                        Ok(record) => record,
                        Err(err) => return Some(Value::error(err, span)),
                    };

                    if let Some(value) = record.get("state") {
                        state = value.clone();
                    }

                    if let Some(out_value) = record.get("out") {
                        Some(out_value.clone())
                    } else {
                        None
                    }
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
