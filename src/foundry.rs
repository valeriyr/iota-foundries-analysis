use std::str::{self, Utf8Error};

use iota_sdk::{
    client::{Client, Result},
    types::block::output::{
        feature::{Irc27Metadata, Irc30Metadata},
        Feature, FoundryOutput, Output,
    },
};

#[derive(Debug)]
pub enum FoundryMetadata {
    None,
    Irc27(Irc27Metadata),
    Irc30(Irc30Metadata),
    DeserializationError {
        str: String,
        irc27: serde_json::Error,
        irc30: serde_json::Error,
    },
    Utf8Error(Vec<u8>, Utf8Error),
}

impl From<&[u8]> for FoundryMetadata {
    fn from(raw: &[u8]) -> Self {
        let utf8_str = str::from_utf8(raw);

        match utf8_str {
            Ok(utf8_str) => {
                let irc27 = serde_json::from_str::<Irc27Metadata>(utf8_str);
                let irc30 = serde_json::from_str::<Irc30Metadata>(utf8_str);

                match irc27 {
                    Ok(irc27) => FoundryMetadata::Irc27(irc27),
                    Err(irc27_e) => match irc30 {
                        Ok(irc30) => FoundryMetadata::Irc30(irc30),
                        Err(irc30_e) => FoundryMetadata::DeserializationError {
                            irc27: irc27_e,
                            irc30: irc30_e,
                            str: utf8_str.to_owned(),
                        },
                    },
                }
            }
            Err(e) => FoundryMetadata::Utf8Error(raw.into(), e),
        }
    }
}

impl From<&FoundryOutput> for FoundryMetadata {
    fn from(output: &FoundryOutput) -> Self {
        for feature in output.immutable_features().iter() {
            if let Feature::Metadata(m) = feature {
                return FoundryMetadata::from(m.data());
            }
        }

        FoundryMetadata::None
    }
}

#[derive(Debug)]
pub struct FoundriesStats {
    pub node_url: String,

    pub number: usize,

    pub with_meta: usize,
    pub without_meta: usize,

    pub irc27: usize,
    pub irc30: usize,

    pub broken_utf8: usize,
    pub deserialization_error: usize,
}

#[derive(Debug)]
pub struct NodeData {
    pub node_url: String,

    pub foundries: Vec<FoundryMetadata>,
}

impl NodeData {
    pub async fn collect(node_url: &str) -> Result<Self> {
        let mut foundries = Vec::new();

        let client = Client::builder().with_node(node_url)?.finish().await?;

        let ids = client.foundry_output_ids([]).await?;

        let outputs = client.get_outputs(&ids).await?;

        for output in outputs {
            match output.output() {
                Output::Foundry(o) => {
                    foundries.push(FoundryMetadata::from(o));
                }
                _ => unreachable!("The output should always be a foundry!"),
            }
        }

        Ok(Self {
            node_url: node_url.to_owned(),
            foundries,
        })
    }

    pub fn stats(&self) -> FoundriesStats {
        let number = self.foundries.len();

        let with_meta = self
            .foundries
            .iter()
            .filter(|m| !matches!(m, FoundryMetadata::None))
            .count();

        let without_meta = number - with_meta;

        let irc27 = self
            .foundries
            .iter()
            .filter(|m| matches!(m, FoundryMetadata::Irc27(_)))
            .count();

        let irc30 = self
            .foundries
            .iter()
            .filter(|m| matches!(m, FoundryMetadata::Irc30(_)))
            .count();

        let broken_utf8 = self
            .foundries
            .iter()
            .filter(|m| matches!(m, FoundryMetadata::Utf8Error(_, _)))
            .count();

        let deserialization_error = self
            .foundries
            .iter()
            .filter(|m| {
                matches!(
                    m,
                    FoundryMetadata::DeserializationError {
                        str: _,
                        irc27: _,
                        irc30: _
                    }
                )
            })
            .count();

        FoundriesStats {
            node_url: self.node_url.clone(),
            number,
            with_meta,
            without_meta,
            irc27,
            irc30,
            broken_utf8,
            deserialization_error,
        }
    }
}
