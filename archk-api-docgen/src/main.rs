use archk::v1::docs::DocumentationObject;
use clap::builder::TypedValueParser;
use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    JSON,
    Markdown,
}
impl From<String> for Format {
    fn from(value: String) -> Self {
        match value.as_str() {
            "json" => Self::JSON,
            "markdown" => Self::Markdown,
            _ => panic!("invalid format value"),
        }
    }
}
impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JSON => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
        }
    }
}

/// API documentation generator for `archk` in different formats
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(
        long,
        default_value_t = Format::JSON,
        value_parser = clap::builder::PossibleValuesParser::new(["json", "markdown"])
            .map(|s| Format::from(s)),
    )]
    format: Format,
}

fn display_ty<'a>(b: &'a DocumentationObject) -> impl std::fmt::Display + 'a {
    struct Container<'a>(&'a DocumentationObject);
    impl<'a> std::fmt::Display for Container<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0.is_may_ignored {
                write!(f, "?")?;
            }
            write!(f, "{}", self.0.name)?;
            if self.0.is_option {
                write!(f, "?")?;
            }
            if self.0.is_array {
                write!(f, "[]")?;
            }
            Ok(())
        }
    }
    Container(b)
}

fn main() {
    let args = Args::parse();

    let endpoints = archk_api::v1::routes::ENDPOINTS;

    match args.format {
        Format::JSON => {
            let res = serde_json::to_string_pretty(endpoints).expect("json");
            println!("{res}");
        }
        Format::Markdown => {
            let mut later_types = Vec::new();
            for endpoint in endpoints {
                later_types.clear();
                println!("## {} `/api/v1{}`", endpoint.method, endpoint.path);
                println!("{}", endpoint.description);

                if let Some(body) = &endpoint.body {
                    println!("### Body");
                    if body.fields.is_empty() {
                        println!("Body type is `{}`.", display_ty(body));
                    } else {
                        println!("| Name | Type | Description |");
                        println!("|------|------|-------------|");
                        for field in body.fields {
                            println!(
                                "| `{}` | `{}` | {} |",
                                field.name,
                                display_ty(&field.documentation),
                                field.documentation.description
                            );
                            if !field.documentation.fields.is_empty() {
                                later_types.push(field);
                            }
                        }
                    }
                }

                if let Some(response) = &endpoint.response {
                    println!("### Response");
                    if response.fields.is_empty() {
                        println!("Response type is `{}`.", display_ty(response));
                    } else {
                        println!("| Name | Type | Description |");
                        println!("|------|------|-------------|");
                        for field in response.fields {
                            println!(
                                "| `{}` | `{}` | {} |",
                                field.name,
                                display_ty(&field.documentation),
                                field.documentation.description
                            );
                            if !field.documentation.fields.is_empty() {
                                later_types.push(field);
                                for field in field
                                    .documentation
                                    .fields
                                    .iter()
                                    .filter(|v| !v.documentation.fields.is_empty())
                                {
                                    later_types.push(field);
                                }
                            }
                        }
                    }
                }

                for ty in later_types.iter() {
                    let ty = &ty.documentation;
                    println!("### Type: `{}`", ty.name);
                    println!("| Name | Type | Description |");
                    println!("|------|------|-------------|");
                    for field in ty.fields {
                        println!(
                            "| `{}` | `{}` | {} |",
                            field.name,
                            display_ty(&field.documentation),
                            field.documentation.description
                        );
                    }
                }
            }
        }
    }
}
