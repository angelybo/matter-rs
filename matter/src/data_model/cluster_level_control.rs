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
    tlv::TLVElement,
};
use log::info;
use num_derive::FromPrimitive;

// ID of base cluster for level control, other specifics are defined for lighting - might need an update in next release
pub const ID: u32 = 0x0008;

// IDs of attributes
pub enum Attributes {
    CurrentLevel = 0x0000,
    RemainingTime = 0x0001,
    MinLevel = 0x0002,
    MaxLevel = 0x0003,
    OnLevel = 0x0011,
    Options = 0x000F,
    StartUpCurrentLevel = 0x4000,
}

enum MoveMode {
    Up = 0x00,
    Down = 0x01,
}

impl MoveMode {
    pub fn from_int(src: u8) -> MoveMode {
        if src == 0x00 {
            MoveMode::Up
        } else {
            MoveMode::Down
        }
    }
}

enum StepMode {
    Up = 0x00,
    Down = 0x01,
}

impl StepMode {
    pub fn from_int(src: u8) -> StepMode {
        if src == 0x00 {
            StepMode::Up
        } else {
            StepMode::Down
        }
    }
}

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

        let attrs = [
            Attribute::new(
                Attributes::CurrentLevel as u16,
                AttrValue::Uint8(0),
                Access::RV,
                Quality::PERSISTENT,
            )?,
            Attribute::new(
                Attributes::OnLevel as u16,
                AttrValue::Uint8(0),
                Access::RV,
                Quality::PERSISTENT,
            )?,
            // Default to 0 and 254 - device might setup it's own min/max stuff
            Attribute::new(
                Attributes::MinLevel as u16,
                AttrValue::Uint8(0),
                Access::RV,
                Quality::PERSISTENT,
            )?,
            Attribute::new(
                Attributes::MaxLevel as u16,
                AttrValue::Uint8(254),
                Access::RV,
                Quality::PERSISTENT,
            )?,
            Attribute::new(
                Attributes::StartUpCurrentLevel as u16,
                AttrValue::Uint8(0x00),
                Access::RV,
                Quality::PERSISTENT,
            )?,
            // Options - probably want a custom type here
        ];

        cluster.base.add_attributes(&attrs)?;
        Ok(cluster)
    }

    // TODO: Move level slowly up to a Min/Max
    fn move_level(&mut self, move_mode: MoveMode, rate: u8) -> Result<(), IMStatusCode> {
        match move_mode {
            MoveMode::Up => {
                info!(
                    "Increasing current level to MAX Level at a rate of: {}",
                    rate
                );

                // TODO: Slowly move our level up in the background.
                Err(IMStatusCode::Sucess)
            }
            MoveMode::Down => {
                info!(
                    "Decreasing current level to Min Level at a rate of: {}",
                    rate
                );
                // TODO: Slowly move our level up in the background.

                Err(IMStatusCode::Sucess)
            }
        }
    }

    // TODO: Maybe handle arithmetic better
    fn step_level(&mut self, step_mode: StepMode, step_size: u8) -> Result<(), IMStatusCode> {
        let old_level = self
            .base
            .read_attribute_raw(Attributes::CurrentLevel as u16)?;
        let mut new_level: u8 = 0;

        match step_mode {
            StepMode::Up => {
                if let AttrValue::Uint8(old) = old_level {
                    new_level = *old + step_size;
                    info!(
                        "Stepping current level up by {} to {}",
                        step_size, new_level
                    );
                }

                self.base
                    .write_attribute_raw(
                        Attributes::CurrentLevel as u16,
                        AttrValue::Uint8(new_level),
                    )
                    .map_err(|_| IMStatusCode::Failure)?;
                Err(IMStatusCode::Sucess)
            }
            StepMode::Down => {
                if let AttrValue::Uint8(old) = old_level {
                    new_level = *old - step_size;
                    info!(
                        "Stepping current level down by {} to {}",
                        step_size, new_level
                    );
                }

                self.base
                    .write_attribute_raw(
                        Attributes::CurrentLevel as u16,
                        AttrValue::Uint8(new_level),
                    )
                    .map_err(|_| IMStatusCode::Failure)?;
                Err(IMStatusCode::Sucess)
            }
        }
    }
}

// Command Handling
impl LevelControlCluster {
    fn handle_move_to_lvl(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        let mut tlv_iterator = cmd_data.enter().ok_or(Error::Invalid)?;

        let new_level = tlv_iterator.next().ok_or(IMStatusCode::InvalidDataType)?;

        // TODO: Process these before updating level
        let _trans_time = tlv_iterator.next().ok_or(IMStatusCode::InvalidDataType)?;
        let _options_mask = tlv_iterator.next().ok_or(IMStatusCode::InvalidDataType)?;
        let _options_override = tlv_iterator.next().ok_or(IMStatusCode::InvalidDataType)?;

        self.base.write_attribute_from_tlv(Attributes::CurrentLevel as u16, &new_level)
    }

    fn handle_move(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        let mut tlv_iterator = cmd_data.enter().ok_or(Error::Invalid)?;

        let move_mode = tlv_iterator.next().ok_or(Error::Invalid)?.u8()?;
        let rate = tlv_iterator.next().ok_or(Error::Invalid)?.u8()?;
        let _options_mask = tlv_iterator.next().ok_or(Error::Invalid)?;
        let _options_override = tlv_iterator.next().ok_or(Error::Invalid)?;

        self.move_level(MoveMode::from_int(move_mode), rate)
    }

    fn handle_stop(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        let mut tlv_iterator = cmd_data.enter().ok_or(Error::Invalid)?;
        let _options_mask = tlv_iterator.next().ok_or(Error::Invalid)?;
        let _options_override = tlv_iterator.next().ok_or(Error::Invalid)?;

        self.base
            .write_attribute_raw(Attributes::RemainingTime as u16, AttrValue::Uint8(0))
            .map_err(|_| IMStatusCode::Failure)?;

        // TODO: Stop any command in progress - implement when we implement progress for commands

        Err(IMStatusCode::Sucess)
    }

    fn handle_step(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        let mut tlv_iterator = cmd_data.enter().ok_or(Error::Invalid)?;

        let step_mode = tlv_iterator.next().ok_or(Error::Invalid)?.u8()?;
        let step_size = tlv_iterator.next().ok_or(Error::Invalid)?.u8()?;
        let _options_mask = tlv_iterator.next().ok_or(Error::Invalid)?;
        let _options_override = tlv_iterator.next().ok_or(Error::Invalid)?;
        
        // TODO: Implement this
        let _transition_time = tlv_iterator.next().ok_or(Error::Invalid)?;
        // self.base
        //     .write_attribute_from_tlv(Attributes::RemainingTime as u16, &transition_time)?;

        let old_level = self.base.read_attribute_raw(Attributes::CurrentLevel as u16)?;

        // self.step_level(StepMode::from_int(step_mode), step_size)?;

        // TODO: Wait before executing? Sleeping this thread seems like a TERRIBLE idea
        // use std::{thread, time};

        // self.base.write_attribute_raw(Attributes::RemainingTime as u16,  AttrValue::Uint16(0))?;
        Err(IMStatusCode::Sucess)
    }

    fn handle_move_to_lvl_with_onoff(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        // todo!();
        Err(IMStatusCode::Sucess)
    }

    fn handle_move_with_onoff(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        // todo!();
        Err(IMStatusCode::Sucess)
    }

    fn handle_step_with_onoff(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        // todo!();
        Err(IMStatusCode::Sucess)
    }

    fn handle_stop_with_onoff(&mut self, cmd_data: &TLVElement) -> Result<(), IMStatusCode> {
        // todo!();
        Err(IMStatusCode::Sucess)
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
            Commands::MoveToLevel => self.handle_move_to_lvl(&cmd_req.data),
            Commands::Move => self.handle_move(&cmd_req.data),
            Commands::Step => self.handle_step(&cmd_req.data),
            Commands::Stop => self.handle_stop(&cmd_req.data),
            Commands::MoveToLevelWithOnOff => self.handle_move_to_lvl_with_onoff(&cmd_req.data),
            Commands::MoveWithOnOff => self.handle_move_with_onoff(&cmd_req.data),
            Commands::StepWithOnOff => self.handle_step_with_onoff(&cmd_req.data),
            Commands::StopWithOnOff => self.handle_stop_with_onoff(&cmd_req.data),
            Commands::MoveToClosestFrequency => Err(IMStatusCode::Sucess),
        }
    }
}
