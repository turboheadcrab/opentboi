//! Mirrors the game's draw list onto a reusable pool of Bevy sprites.

use bevy::prelude::*;

use super::GameSimulation;
use super::texture_library::TextureLibrary;
use crate::game::{DrawItem, Rgba};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncSprites;

/// Draw list buffer and the sprite entity drawing each of its slots, reused across frames.
#[derive(Resource, Default)]
struct SpritePool {
    draw_items: Vec<DrawItem>,
    sprite_entities: Vec<Entity>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SpritePool>()
        .add_systems(Update, sync_sprites.in_set(SyncSprites));
}

fn sync_sprites(
    mut commands: Commands,
    simulation: Res<GameSimulation>,
    mut textures: ResMut<TextureLibrary>,
    mut images: ResMut<Assets<Image>>,
    mut pool: ResMut<SpritePool>,
    mut sprites: Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
) {
    let SpritePool {
        draw_items,
        sprite_entities,
    } = &mut *pool;
    draw_items.clear();
    simulation.0.fill_draw_items(draw_items);

    for (slot, draw_item) in draw_items.iter().enumerate() {
        let existing = sprite_entities
            .get(slot)
            .and_then(|&entity| sprites.get_mut(entity).ok());
        match existing {
            Some((mut sprite, mut transform, mut visibility)) => {
                *sprite = sprite_for(draw_item, &mut textures, &mut images);
                *transform = transform_for(draw_item);
                *visibility = Visibility::Inherited;
            }
            None if slot >= sprite_entities.len() => {
                let entity = commands
                    .spawn((
                        sprite_for(draw_item, &mut textures, &mut images),
                        transform_for(draw_item),
                    ))
                    .id();
                sprite_entities.push(entity);
            }
            // Spawned last frame; its components become queryable next frame.
            None => {}
        }
    }

    for &unused_entity in sprite_entities.iter().skip(draw_items.len()) {
        if let Ok((_, _, mut visibility)) = sprites.get_mut(unused_entity) {
            *visibility = Visibility::Hidden;
        }
    }
    let mut upcoming = Vec::new();
    simulation.0.upcoming_textures(&mut upcoming);
    textures.prefetch(&upcoming, &mut images);
}

/// Textures are drawn untinted; items whose texture was never loaded become a rectangle of their
/// fallback color.
fn sprite_for(draw_item: &DrawItem, textures: &mut TextureLibrary, images: &mut Assets<Image>) -> Sprite {
    let loaded_image = draw_item
        .texture
        .and_then(|texture_id| textures.handle(texture_id, images));
    let color = if loaded_image.is_some() {
        Color::WHITE
    } else {
        to_bevy_color(draw_item.color)
    };
    Sprite {
        image: loaded_image.unwrap_or_default(),
        color,
        custom_size: Some(draw_item.size),
        flip_x: draw_item.flip_horizontally,
        ..Sprite::default()
    }
}

fn transform_for(draw_item: &DrawItem) -> Transform {
    Transform::from_translation(draw_item.center.extend(draw_item.depth))
        .with_rotation(Quat::from_rotation_z(draw_item.rotation_radians))
}

fn to_bevy_color(color: Rgba) -> Color {
    Color::srgba(color.red, color.green, color.blue, color.alpha)
}
