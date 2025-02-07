// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

use indexmap::IndexMap;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ping<N: Network> {
    pub version: u32,
    pub node_type: NodeType,
    pub block_locators: Option<BlockLocators<N>>,
}

impl<N: Network> MessageTrait for Ping<N> {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "Ping".into()
    }

    /// Serializes the message into the buffer.
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.version.write_le(&mut *writer)?;
        self.node_type.write_le(&mut *writer)?;
        if let Some(locators) = &self.block_locators {
            1u8.write_le(&mut *writer)?;

            (locators.recents.len().min(u32::MAX as usize) as u32).write_le(&mut *writer)?;
            for (height, hash) in locators.recents.iter() {
                height.write_le(&mut *writer)?;
                hash.write_le(&mut *writer)?;
            }

            (locators.checkpoints.len().min(u32::MAX as usize) as u32).write_le(&mut *writer)?;
            for (height, hash) in locators.checkpoints.iter() {
                height.write_le(&mut *writer)?;
                hash.write_le(&mut *writer)?;
            }
        } else {
            0u8.write_le(&mut *writer)?;
        }

        Ok(())
    }

    /// Deserializes the given buffer into a message.
    #[inline]
    fn deserialize(bytes: BytesMut) -> Result<Self> {
        let mut reader = bytes.reader();

        let version = u32::read_le(&mut reader)?;
        let node_type = NodeType::read_le(&mut reader)?;

        if u8::read_le(&mut reader)? == 0 {
            return Ok(Self { version, node_type, block_locators: None });
        }

        let mut recents = IndexMap::new();
        let num_recents = u32::read_le(&mut reader)?;
        for _ in 0..num_recents {
            let height = u32::read_le(&mut reader)?;
            let hash = N::BlockHash::read_le(&mut reader)?;
            recents.insert(height, hash);
        }

        let mut checkpoints = IndexMap::new();
        let num_checkpoints = u32::read_le(&mut reader)?;
        for _ in 0..num_checkpoints {
            let height = u32::read_le(&mut reader)?;
            let hash = N::BlockHash::read_le(&mut reader)?;
            checkpoints.insert(height, hash);
        }

        let block_locators = Some(BlockLocators { recents, checkpoints });

        Ok(Self { version, node_type, block_locators })
    }
}

impl<N: Network> Ping<N> {
    pub fn new(node_type: NodeType, block_locators: Option<BlockLocators<N>>) -> Self {
        Self { version: <Message<N>>::VERSION, node_type, block_locators }
    }
}
