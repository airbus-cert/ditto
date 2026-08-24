use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::name::{NameGenerator, VerbsNameGenerator};
use crate::pass::Pass;
use crate::tree::Node;
use indoc::indoc;
use rand::prelude::*;
use rand::rngs::ThreadRng;

static SNIPPETS: &'static [&'static str] = &[
    indoc! { r#"
        function RANDOM_STR {
            param (
                [string]$arg
            )
            Write-Host "RANDOM_STR"
        }
    "#},
    indoc! { r#"
        for ($i = 0; $i -lt RANDOM_INT; $i++) {
            $i | Out-Null
        }
    "#},
    indoc! { r#"
        Start-Job -ScriptBlock {
            $null | Out-Null
        } | Out-Null
    "#},
    indoc! { r#"
        ${RANDOM_STR} = @()
        ${RANDOM_STR} += RANDOM_INT
    "#},
    indoc! { r#"
        $tempFile = "C:\Temp\RANDOM_STR.txt"
        New-Item -Path $tempFile -ItemType File -Force | Out-Null
        Remove-Item -Path $tempFile -Force | Out-Null
    "#},
    indoc! { r#"
        try{
            Remove-Item -Path ${RANDOM_STR} -Force | Out-Null
        } catch {

        }
    "#},
    "Get-Process | Out-Null",
];

#[derive(Default)]
pub struct BetweenDeadCode<T: Default> {
    rng: ThreadRng,
    generator: T,
}

impl<T: Default + NameGenerator> BetweenDeadCode<T> {
    fn generate_dummy_snippet(&mut self) -> String {
        let mut snippet = SNIPPETS[self.rng.random_range(0..SNIPPETS.len() - 1)].to_string();
        snippet = snippet.replace("RANDOM_STR", &self.generator.generate(false));
        snippet = snippet.replace("RANDOM_INT", &self.rng.random_range(0..0x100).to_string());

        snippet
    }

    fn generate_dummy_snippet_with_size(&mut self, min_size: usize) -> String {
        let mut snippet = String::from("");

        while snippet.len() < min_size {
            snippet += &self.generate_dummy_snippet();
        }

        snippet
    }
}

impl<T: NameGenerator + Default> Pass for BetweenDeadCode<T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "program" {
            self.rng = rand::rng();
            let text = node.text()?;

            let mut new_node = self.generate_dummy_snippet_with_size(0x2000);
            new_node += text;
            if let Some(last) = text.chars().last() {
                if last != '\n' {
                    new_node += "\n";
                }
            }
            new_node += &self.generate_dummy_snippet_with_size(0x2000);

            Ok(Some(vec![Mutation::substitute(node, new_node)?]))
        } else {
            Ok(None)
        }
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type BetweenVerbDeadCode = BetweenDeadCode<VerbsNameGenerator>;
