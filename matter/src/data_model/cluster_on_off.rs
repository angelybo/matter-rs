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
    cmd_enter,
    error::*,
    interaction_model::{command::CommandReq, core::IMStatusCode},
};
use log::info;
use num_derive::FromPrimitive;
use std::sync::{Arc, Mutex};

pub const ID: u32 = 0x0006;

pub enum Attributes {
    OnOff = 0x0,
}

#[derive(FromPrimitive, PartialEq)]
pub enum Commands {
    Off = 0x0,
    On = 0x01,
    Toggle = 0x02,
}

fn attr_on_off_new() -> Result<Attribute, Error> {
    // OnOff, Value: false
    Attribute::new(
        Attributes::OnOff as u16,
        AttrValue::Bool(false),
        Access::RV,
        Quality::PERSISTENT,
    )
}

struct ClusterCallback {
    name: Commands,
    callback: Box<dyn FnMut()>,
}

pub struct UpdateData {
    on_off: bool,
    is_fresh: bool
}

impl UpdateData {
    pub fn update_state(&mut self, state: bool) {
        self.on_off = state;
        self.is_fresh = true;
    }
}

pub struct OnOffCluster {
    base: Cluster,
    callbacks: Vec<ClusterCallback>,
    update_state: Arc<Mutex<UpdateData>>
}

impl OnOffCluster {
    pub fn new() -> Result<Box<Self>, Error> {
        let mut cluster = Box::new(OnOffCluster {
            base: Cluster::new(ID)?,
            callbacks: vec!(),
            update_state: Arc::new(Mutex::new(UpdateData { on_off: false, is_fresh: false }))
        });
        cluster.base.add_attribute(attr_on_off_new()?)?;
        Ok(cluster)
    }

    pub fn add_callback(&mut self, cmd: Commands, cb: Box<dyn FnMut()> ) {
        self.callbacks.push(ClusterCallback{ name: cmd, callback: cb });
    }

    pub fn run_callback(&mut self, cmd: Commands) {
        for cb in self.callbacks.iter_mut() {
            if cb.name == cmd {
                (cb.callback)();
            }   
        }
    }
}

impl ClusterType for OnOffCluster {
    fn base(&self) -> &Cluster {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Cluster {
        &mut self.base
    }

    fn read_attribute(
        &self,
        access_req: &mut crate::acl::AccessReq,
        encoder: &mut dyn Encoder,
        attr: &AttrDetails,
    ) {
        let mut error = IMStatusCode::Sucess;
        let base = self.base();
        let a = if let Ok(a) = base.get_attribute(attr.attr_id) {
            a
        } else {
            encoder.encode_status(IMStatusCode::UnsupportedAttribute, 0);
            return;
        };
        if !a.access.contains(Access::READ) {
            error = IMStatusCode::UnsupportedRead;
        }
        access_req.set_target_perms(a.access);
        if !access_req.allow() {
            error = IMStatusCode::UnsupportedAccess;
        }
        if error != IMStatusCode::Sucess {
            encoder.encode_status(error, 0);
        } else if Attribute::is_system_attr(attr.attr_id) {
            self.base().read_system_attribute(encoder, a)
        } else if a.value != AttrValue::Custom {
            // Read data from event loop
            let update_state = self.update_state.lock().unwrap();

            if update_state.is_fresh {
                let val = AttrValue::Bool(update_state.on_off);
                encoder.encode(EncodeValue::Value(&val))
            } else {
                encoder.encode(EncodeValue::Value(&a.value))
            }
        } else {
            self.read_custom_attribute(encoder, attr)
        }
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
            Commands::Off => {
                cmd_enter!("Off");
                let value = self
                    .base
                    .read_attribute_raw(Attributes::OnOff as u16)
                    .unwrap();
                if AttrValue::Bool(true) == *value {
                    self.base
                        .write_attribute_raw(Attributes::OnOff as u16, AttrValue::Bool(false))
                        .map_err(|_| IMStatusCode::Failure)?;
                }

                self.run_callback(Commands::Off);
                cmd_req.trans.complete();
                Err(IMStatusCode::Sucess)
            }
            Commands::On => {
                cmd_enter!("On");
                let value = self
                    .base
                    .read_attribute_raw(Attributes::OnOff as u16)
                    .unwrap();
                if AttrValue::Bool(false) == *value {
                    self.base
                        .write_attribute_raw(Attributes::OnOff as u16, AttrValue::Bool(true))
                        .map_err(|_| IMStatusCode::Failure)?;
                }

                self.run_callback(Commands::On);
                cmd_req.trans.complete();
                Err(IMStatusCode::Sucess)
            }
            Commands::Toggle => {
                cmd_enter!("Toggle");
                let value = match self
                    .base
                    .read_attribute_raw(Attributes::OnOff as u16)
                    .unwrap()
                {
                    &AttrValue::Bool(v) => v,
                    _ => false,
                };
                self.base
                    .write_attribute_raw(Attributes::OnOff as u16, AttrValue::Bool(!value))
                    .map_err(|_| IMStatusCode::Failure)?;
                
                self.run_callback(Commands::Toggle);
                cmd_req.trans.complete();
                Err(IMStatusCode::Sucess)
            }
        }
    }
}
