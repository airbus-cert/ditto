use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::pass::Pass;
use crate::tree::Node;
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Default, Debug)]
pub struct Base64Str;

impl Pass for Base64Str {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "verbatim_string_characters" {
            let str = String::from(node.text()?);
            let str_without_quote = String::from(&str[1..str.len() - 1]);
            let encoded = STANDARD.encode(str_without_quote);
            Ok(Some(vec![Mutation::substitute(
                node,
                "([System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String('"
                    .to_string()
                    + &encoded
                    + "')))",
            )?]))
        } else {
            Ok(None)
        }
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

#[derive(Default, Debug)]
pub struct UUIDStr;

impl Pass for UUIDStr {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "verbatim_string_characters" => {
                let str = String::from(node.text()?);

                // Skip too big strings as it may create enormous UUID array
                // if str.len() > 16 * 100 {
                //     return Ok(None);
                // }

                let str_without_quote = String::from(&str[1..str.len() - 1]);
                let encoded: Vec<String> = str_without_quote
                    .as_bytes()
                    .chunks(16)
                    .map(|chunk| {
                        let hex = chunk
                            .iter()
                            .fold("".to_string(), |acc, b| acc + &format!("{:x}", b));
                        let padded = format!("{:0<32}", hex);

                        format!(
                            "{}-{}-{}-{}-{}",
                            &padded[..8],
                            &padded[8..12],
                            &padded[12..16],
                            &padded[16..20],
                            &padded[20..]
                        )
                    })
                    .collect();
                let psh_array = format!(
                    "@({})",
                    encoded
                        .into_iter()
                        .map(|u| format!("'{}'", u))
                        .collect::<Vec<String>>()
                        .join(",")
                );
                let x = format!(
                    "(-join (-join {} | % {{$_ -replace '00|-'}} | % {{$_ -split '(..)'}} | ? {{ $_ }} | % {{ [char][convert]::ToUInt32($_,16) }}))",
                    psh_array
                );
                Ok(Some(vec![Mutation::substitute(node, x)?]))
            }
            _ => Ok(None),
        }
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
