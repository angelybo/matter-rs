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

mod dev_att;
use matter::core::{self, CommissioningData};
use matter::data_model::cluster_basic_information::BasicInfoConfig;
use matter::data_model::cluster_level_control::{LevelControlCluster, Commands as LvlCommands} ;
use matter::data_model::cluster_media_playback::{MediaPlaybackCluster, Commands as MediaCommands};
use matter::data_model::device_types::DEV_TYPE_ON_SMART_SPEAKER;
use matter::secure_channel::spake2p::VerifierData;
use log::{info,debug, error};

fn setup_media_playback_callbacks(media_playback_cluster: &mut Box<MediaPlaybackCluster>) {
    let play_callback = Box::new(|| info!("Comamnd [Play] handled with callback."));
    let pause_callback = Box::new(|| info!("Comamnd [Pause] handled with callback."));
    let stop_callback = Box::new(|| info!("Comamnd [Stop] handled with callback."));
    let start_over_callback =
        Box::new(|| info!("Comamnd [StartOver] handled with callback."));


    media_playback_cluster.add_callback(MediaCommands::Play, play_callback);
    media_playback_cluster.add_callback(MediaCommands::Pause, pause_callback);
    media_playback_cluster.add_callback(MediaCommands::Stop, stop_callback);
    media_playback_cluster.add_callback(MediaCommands::StartOver, start_over_callback);
}

fn setup_level_control_callbacks(level_control_cluster: &mut Box<LevelControlCluster>) {
    let move_to_lvl_callback = Box::new(|a,b,c| info!("Command [MoveToLevel] handled."));
    let move_callback = Box::new(|_,_,_| info!("Command [Move] handled."));
    let step_callback = Box::new(|_,_,_| info!("Command [Step] handled."));
    let stop_callback = Box::new(|_,_,_| info!("Command [Stop] handled."));

    level_control_cluster.add_data_callback(LvlCommands::MoveToLevel, move_to_lvl_callback);
    level_control_cluster.add_data_callback(LvlCommands::Move, move_callback);
    level_control_cluster.add_data_callback(LvlCommands::Step, step_callback);
    level_control_cluster.add_data_callback(LvlCommands::Stop, stop_callback);
}


fn main() {
    env_logger::init();
    let comm_data = CommissioningData {
        // TODO: Hard-coded for now
        verifier: VerifierData::new_with_pw(123456),
        discriminator: 250,
    };

    // vid/pid should match those in the DAC
    let dev_info = BasicInfoConfig {
        vid: 0xFFF1,
        pid: 0x8002,
        hw_ver: 2,
        sw_ver: 1,
        sw_ver_str: "1".to_string(),
        serial_no: "aabbccdd".to_string(),
        device_name: "Smart Speaker".to_string(),
    };
    let dev_att = Box::new(dev_att::HardCodedDevAtt::new());

    let mut matter = core::Matter::new(dev_info, dev_att, comm_data).unwrap();
    let dm = matter.get_data_model();
    {
        let mut node = dm.node.write().unwrap();

        let endpoint_audio = node.add_endpoint(DEV_TYPE_ON_SMART_SPEAKER).unwrap();
        let mut lvl_control_cluster = match LevelControlCluster::new() {
            Ok(t) => t,
            Err(_) => {
                error!("Failed to allocate LevelControlCluster");
                panic!("Failed to allocate MediaPlaybackCluster");
            },
        };
        let mut media_playback_cluster = match MediaPlaybackCluster::new() {
            Ok(t) => t,
            Err(_) => {
                error!("Failed to allocate MediaPlaybackCluster");
                panic!("Failed to allocate MediaPlaybackCluster");
            }
        };

        setup_media_playback_callbacks(&mut media_playback_cluster);
        setup_level_control_callbacks(&mut lvl_control_cluster);

        match node.add_cluster(endpoint_audio, lvl_control_cluster) {
            Ok(t) =>  {
                debug!("Added level control cluster to node");
            },
            Err(_) => {
                error!("Failed to add cluster to node: Too many clusters");
            }
        };
        match node.add_cluster(endpoint_audio, media_playback_cluster) {
            Ok(t) => {
                debug!("Added level control cluster to node");
            },
            Err(_) => {
                error!("Failed to add cluster to node: Too many clusters");
            }
        };

        println!("Added Speaker type at endpoint id: {}", endpoint_audio);
    }
    matter.start_daemon().unwrap();
}
