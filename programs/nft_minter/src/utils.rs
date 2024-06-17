use anchor_lang::prelude::*;
use mpl_token_metadata::state::Creator as MPL_Creator;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

impl From<MPL_Creator> for Creator {
    fn from(item: MPL_Creator) -> Self {
        Creator {
            address: item.address,
            verified: item.verified,
            share: item.share,
        }
    }
}

pub fn into_mpl_creators(creators: Option<Vec<Creator>>) -> Option<Vec<MPL_Creator>> {
    creators.map(|creators| {
        creators
            .iter()
            .map(|creator| MPL_Creator {
                address: creator.address,
                share: creator.share,
                verified: creator.verified,
            })
            .collect()
    })
}

pub fn token_metadata_program_id() -> Pubkey {
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        .parse()
        .unwrap()
}
