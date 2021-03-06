use crate::commands::WholeStreamCommand;
use crate::prelude::*;
use nu_errors::ShellError;
use nu_protocol::{ColumnPath, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use nu_source::{Tag, Tagged};
use nu_value_ext::ValueExt;

#[derive(Deserialize)]
struct Arguments {
    replace: Tagged<String>,
    rest: Vec<ColumnPath>,
}

pub struct SubCommand;

#[async_trait]
impl WholeStreamCommand for SubCommand {
    fn name(&self) -> &str {
        "str set"
    }

    fn signature(&self) -> Signature {
        Signature::build("str set")
            .required("set", SyntaxShape::String, "the new string to set")
            .rest(
                SyntaxShape::ColumnPath,
                "optionally set text by column paths",
            )
    }

    fn usage(&self) -> &str {
        "sets text"
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        operate(args, registry)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Set contents with preferred string",
                example: "echo 'good day' | str set 'good bye'",
                result: Some(vec![Value::from("good bye")]),
            },
            Example {
                description: "Set the contents on preferred column paths",
                example: "open Cargo.toml | str set '255' package.version",
                result: None,
            },
        ]
    }
}

#[derive(Clone)]
struct Replace(String);

fn operate(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();

    let stream = async_stream! {
        let (Arguments { replace, rest }, mut input) = args.process(&registry).await?;
        let options = Replace(replace.item);

        let column_paths: Vec<_> = rest.iter().map(|x| x.clone()).collect();

        while let Some(v) = input.next().await {
            if column_paths.is_empty() {
                match action(&v, &options, v.tag()) {
                    Ok(out) => yield ReturnSuccess::value(out),
                    Err(err) => {
                        yield Err(err);
                        return;
                    }
                }
            } else {

                let mut ret = v.clone();

                for path in &column_paths {
                    let options = options.clone();

                    let swapping = ret.swap_data_by_column_path(path, Box::new(move |old| action(old, &options, old.tag())));

                    match swapping {
                        Ok(new_value) => {
                            ret = new_value;
                        }
                        Err(err) => {
                            yield Err(err);
                            return;
                        }
                    }
                }

                yield ReturnSuccess::value(ret);
            }
        }
    };

    Ok(stream.to_output_stream())
}

fn action(_input: &Value, options: &Replace, tag: impl Into<Tag>) -> Result<Value, ShellError> {
    let replacement = &options.0;
    Ok(UntaggedValue::string(replacement.as_str()).into_value(tag))
}

#[cfg(test)]
mod tests {
    use super::{action, Replace, SubCommand};
    use nu_plugin::test_helpers::value::string;
    use nu_source::Tag;

    #[test]
    fn examples_work_as_expected() {
        use crate::examples::test as test_examples;

        test_examples(SubCommand {})
    }

    #[test]
    fn sets() {
        let word = string("andres");
        let expected = string("robalino");

        let set_options = Replace(String::from("robalino"));

        let actual = action(&word, &set_options, Tag::unknown()).unwrap();
        assert_eq!(actual, expected);
    }
}
