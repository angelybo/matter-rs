/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use super::objects::*;
use crate::{
    error::*,
    interaction_model::{command::CommandReq, core::IMStatusCode},
};
use num_derive::FromPrimitive;

// ID of base cluster for level control, other specifics are defined for lighting - might need an update in next release
pub const ID: u32 = 0x0008;

// IDs of attributes
pub enum Attributes {
    CurrentLevel = 0x0,
    OnLevel = 0x0011,
    Options = 0x000F,
}

// TODO: Depricate these when add bulk attributes is merged
fn attr_current_level_new() -> Result<Attribute, Error> {
    Attribute::new(
        Attributes::CurrentLevel as u16,
        AttrValue::Uint8(0),
        Access::RV,
        Quality::PERSISTENT,
    )
}

fn attr_on_level_new() -> Result<Attribute, Error> {
    Attribute::new(
        Attributes::OnLevel as u16,
        AttrValue::Uint8(0),
        Access::RV,
        Quality::PERSISTENT,
    )
}

// TODO: Implement Options using a map8 type

#[derive(FromPrimitive)]
pub enum Commands {
    MoveToLevel = 0x00,
    Move = 0x01,
    Step = 0x02,
    Stop = 0x03,
    MoveToLevelWithOnOff = 0x04,
    MoveWithOnOff = 0x05,
    StepWithOnOff = 0x06,
    StopWithOnOff = 0x07,
    MoveToClosestFrequency = 0x08,
}

pub struct LevelControlCluster {
    base: Cluster,
}

impl LevelControlCluster {
    pub fn new() -> Result<Box<Self>, Error> {
        let mut cluster = Box::new(LevelControlCluster {
            base: Cluster::new(ID)?,
        });

        cluster.base.add_attribute(attr_current_level_new()?)?;
        cluster.base.add_attribute(attr_on_level_new()?)?;

        Ok(cluster)
    }
}

impl ClusterType for LevelControlCluster {
    fn base(&self) -> &Cluster {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Cluster {
        &mut self.base
    }

    fn handle_command(&mut self, cmd_req: &mut CommandReq) -> Result<(), IMStatusCode> {
        let cmd = cmd_req
            .cmd
            .path
            .leaf
            .map(num::FromPrimitive::from_u32)
            .ok_or(IMStatusCode::UnsupportedCommand)?
            .ok_or(IMStatusCode::UnsupportedCommand)?;
        match cmd {
            Commands::MoveToLevel => {


                // TODO: Get data from cmdReq.data

                let cmd_tlv_elements = cmd_req.data;

                let cmd_data_type = cmd_tlv_elements.get_element_type();

                let new_level = 0;

                self.base
                    .write_attribute_raw(
                        Attributes::CurrentLevel as u16,
                        AttrValue::Uint8(new_level),
                    )
                    .map_err(|_| IMStatusCode::Failure)?;

                Err(IMStatusCode::Sucess)
            }

            Commands::Move => Err(IMStatusCode::Sucess),
            Commands::Step => Err(IMStatusCode::Sucess),
            Commands::Stop => Err(IMStatusCode::Sucess),
            Commands::MoveToLevelWithOnOff => Err(IMStatusCode::Sucess),
            Commands::MoveWithOnOff => Err(IMStatusCode::Sucess),
            Commands::StepWithOnOff => Err(IMStatusCode::Sucess),
            Commands::StopWithOnOff => Err(IMStatusCode::Sucess),
            Commands::MoveToClosestFrequency => Err(IMStatusCode::Sucess),
        }
    }
}

// Command implementations

// TODO: Implement OptionMask and OptionOverride
pub struct MoveToLevelCommand {
    Level: u8,
    TransitionTime: u16,
}

impl MoveToLevelCommand {
    pub fn new() -> Result<Box<Self>, Error> {
        let command = Box::new(MoveToLevelCommand {
            Level: 0,
            TransitionTime: 0,
        });

        Ok(command)
    }


    pub fn from_raw(level: u8, TransitionTime: u16)  -> Result<Box<Self>, Error>  {
        let mut command = Box::new(MoveToLevelCommand {
            Level: level,
            TransitionTime: TransitionTime
        });
        Ok(command)
    }
}
