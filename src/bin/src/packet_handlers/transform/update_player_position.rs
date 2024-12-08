use ferrumc_core::chunks::chunk_receiver::ChunkReceiver;
use ferrumc_core::transform::grounded::OnGround;
use ferrumc_core::transform::position::Position;
use ferrumc_core::transform::rotation::Rotation;
use ferrumc_macros::event_handler;
use ferrumc_net::errors::NetError;
use ferrumc_net::packets::packet_events::TransformEvent;
use ferrumc_net::utils::ecs_helpers::EntityExt;
use ferrumc_state::GlobalState;

#[event_handler]
async fn handle_player_move(
    event: TransformEvent,
    state: GlobalState,
) -> Result<TransformEvent, NetError> {
    let conn_id = event.conn_id;
    let mut calculate_chunks = false;
    if let Some(ref new_position) = event.position {
        let mut position = conn_id.get_mut::<Position>(&state)?;

        let mut chunk_recv = state.universe.get_mut::<ChunkReceiver>(conn_id)?;
        if let Some(last_chunk) = &chunk_recv.last_chunk {
            let new_chunk = (
                new_position.x as i32 / 16,
                new_position.z as i32 / 16,
                String::from("overworld"),
            );
            if *last_chunk != new_chunk {
                chunk_recv.last_chunk = Some(new_chunk);
                calculate_chunks = true;
            }
        } else {
            chunk_recv.last_chunk = Some((
                new_position.x as i32 / 16,
                new_position.z as i32 / 16,
                String::from("overworld"),
            ));
            calculate_chunks = true;
        }

        *position = Position::new(new_position.x, new_position.y, new_position.z);
    }

    if let Some(ref new_rotation) = event.rotation {
        let mut rotation = conn_id.get_mut::<Rotation>(&state)?;

        *rotation = Rotation::new(new_rotation.yaw, new_rotation.pitch);
    }

    if let Some(new_grounded) = event.on_ground {
        let mut on_ground = conn_id.get_mut::<OnGround>(&state)?;

        *on_ground = OnGround(new_grounded);
    }

    if calculate_chunks {
        let chunk_recv = state.universe.get_mut::<ChunkReceiver>(conn_id)?;
        chunk_recv.can_see.clear().await;
        let (center_x, center_z, dimension) = chunk_recv.last_chunk.as_ref().unwrap();
        for x in center_x - crate::VIEW_DISTANCE..=center_x + crate::VIEW_DISTANCE {
            for z in center_z - crate::VIEW_DISTANCE..=center_z + crate::VIEW_DISTANCE {
                chunk_recv.needed_chunks.insert((x, z, dimension.clone()), None).await;
                chunk_recv.can_see.insert((x, z, dimension.clone())).await;
            }
        }
    }

    Ok(event)
}
