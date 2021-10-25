use nu_engine::{eval_block, eval_expression};
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Example, IntoPipelineData, PipelineData, Signature, Span, SyntaxShape, Value};

#[derive(Clone)]
pub struct For;

impl Command for For {
    fn name(&self) -> &str {
        "for"
    }

    fn usage(&self) -> &str {
        "Loop over a range"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("for")
            .required(
                "var_name",
                SyntaxShape::VarWithOptType,
                "name of the looping variable",
            )
            .required(
                "range",
                SyntaxShape::Keyword(b"in".to_vec(), Box::new(SyntaxShape::Any)),
                "range of the loop",
            )
            .required(
                "block",
                SyntaxShape::Block(Some(vec![])),
                "the block to run",
            )
            .creates_scope()
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let var_id = call.positional[0]
            .as_var()
            .expect("internal error: missing variable");

        let keyword_expr = call.positional[1]
            .as_keyword()
            .expect("internal error: missing keyword");
        let values = eval_expression(engine_state, stack, keyword_expr)?;

        let block = call.positional[2]
            .as_block()
            .expect("internal error: expected block");

        let engine_state = engine_state.clone();
        let stack = stack.enter_scope();

        match values {
            Value::List { vals, span } => Ok(vals
                .into_iter()
                .map(move |x| {
                    let block = engine_state.get_block(block);

                    let mut stack = stack.clone();
                    stack.add_var(var_id, x);

                    match eval_block(&engine_state, &mut stack, block, PipelineData::new()) {
                        Ok(value) => Value::List {
                            vals: value.collect(),
                            span,
                        },
                        Err(error) => Value::Error { error },
                    }
                })
                .into_pipeline_data()),
            Value::Range { val, span } => Ok(val
                .into_range_iter()?
                .map(move |x| {
                    let block = engine_state.get_block(block);

                    let mut stack = stack.enter_scope();

                    stack.add_var(var_id, x);

                    match eval_block(&engine_state, &mut stack, block, PipelineData::new()) {
                        Ok(value) => Value::List {
                            vals: value.collect(),
                            span,
                        },
                        Err(error) => Value::Error { error },
                    }
                })
                .into_pipeline_data()),
            x => {
                let block = engine_state.get_block(block);

                let mut stack = stack.enter_scope();

                stack.add_var(var_id, x);

                eval_block(&engine_state, &mut stack, block, PipelineData::new())
            }
        }
    }

    fn examples(&self) -> Vec<Example> {
        let span = Span::unknown();
        vec![
            Example {
                description: "Echo the square of each integer",
                example: "for x in [1 2 3] { $x * $x }",
                result: Some(Value::List {
                    vals: vec![
                        Value::Int { val: 1, span },
                        Value::Int { val: 4, span },
                        Value::Int { val: 9, span },
                    ],
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "Work with elements of a range",
                example: "for $x in 1..3 { $x }",
                result: Some(Value::List {
                    vals: vec![
                        Value::Int { val: 1, span },
                        Value::Int { val: 2, span },
                        Value::Int { val: 3, span },
                    ],
                    span: Span::unknown(),
                }),
            },
            // FIXME? Numbered `for` is kinda strange, but was supported in previous nushell
            // Example {
            //     description: "Number each item and echo a message",
            //     example: "for $it in ['bob' 'fred'] --numbered { $\"($it.index) is ($it.item)\" }",
            //     result: Some(Value::List {
            //         vals: vec![
            //             Value::String {
            //                 val: "0 is bob".into(),
            //                 span,
            //             },
            //             Value::String {
            //                 val: "0 is fred".into(),
            //                 span,
            //             },
            //         ],
            //         span: Span::unknown(),
            //     }),
            // },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(For {})
    }
}
