use entity_table::ComponentTable;
#[cfg(feature = "serialize")]
use entity_table::ComponentTableEntries;
pub use entity_table::Entity; // public so it can be referenced in macro body
use grid_2d::Grid;
pub use grid_2d::{Coord, Size};
#[cfg(feature = "serialize")]
pub use serde; // public so it can be referenced in macro body
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub trait Layers: Default {
    type Layer: Copy;
    fn select_field_mut(&mut self, layer: Self::Layer) -> &mut Option<Entity>;
}

#[cfg(not(feature = "serialize"))]
#[macro_export]
macro_rules! declare_layers_module {
    { $module_name:ident { $($field_name:ident: $variant_name:ident,)* } } => {
        mod $module_name {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct Layers {
                $(pub $field_name: Option<$crate::Entity>,)*
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Layer {
                $($variant_name,)*
            }

            impl Default for Layers {
                fn default() -> Self {
                    Self {
                        $($field_name: None,)*
                    }
                }
            }

            impl $crate::Layers for Layers {
                type Layer = Layer;
                fn select_field_mut(&mut self, layer: Self::Layer) -> &mut Option<$crate::Entity> {
                    match layer {
                        $(Layer::$variant_name => &mut self.$field_name,)*
                    }
                }
            }
        }
    }
}

#[cfg(feature = "serialize")]
#[macro_export]
macro_rules! declare_layers_module {
    { $module_name:ident { $($field_name:ident: $variant_name:ident,)* } } => {
        mod $module_name {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, $crate::serde::Serialize, $crate::serde::Deserialize)]
            pub struct Layers {
                $(pub $field_name: Option<$crate::Entity>,)*
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, $crate::serde::Serialize, $crate::serde::Deserialize)]
            pub enum Layer {
                $($variant_name,)*
            }

            impl Default for Layers {
                fn default() -> Self {
                    Self {
                        $($field_name: None,)*
                    }
                }
            }

            impl $crate::Layers for Layers {
                type Layer = Layer;
                fn select_field_mut(&mut self, layer: Self::Layer) -> &mut Option<$crate::Entity> {
                    match layer {
                        $(Layer::$variant_name => &mut self.$field_name,)*
                    }
                }
            }
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location<L> {
    pub coord: Coord,
    pub layer: Option<L>,
}

#[derive(Debug)]
pub struct SpatialTable<L: Layers> {
    location_component: ComponentTable<Location<L::Layer>>,
    spatial_grid: Grid<L>,
}

pub type Enumerate<'a, L> = grid_2d::GridEnumerate<'a, L>;

impl<L: Layers> SpatialTable<L> {
    pub fn new(size: Size) -> Self {
        let location_component = ComponentTable::default();
        let spatial_grid = Grid::new_default(size);
        Self {
            location_component,
            spatial_grid,
        }
    }
    pub fn enumerate(&self) -> Enumerate<L> {
        self.spatial_grid.enumerate()
    }
    pub fn grid_size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn layers_at(&self, coord: Coord) -> Option<&L> {
        self.spatial_grid.get(coord)
    }
    pub fn layers_at_checked(&self, coord: Coord) -> &L {
        self.spatial_grid.get_checked(coord)
    }
    pub fn location_of(&self, entity: Entity) -> Option<&Location<L::Layer>> {
        self.location_component.get(entity)
    }
    pub fn coord_of(&self, entity: Entity) -> Option<Coord> {
        self.location_of(entity).map(|l| l.coord)
    }
    pub fn update(
        &mut self,
        entity: Entity,
        location: Location<L::Layer>,
    ) -> Result<(), UpdateError> {
        if let Some(layer) = location.layer {
            let cell = self
                .spatial_grid
                .get_mut(location.coord)
                .ok_or(UpdateError::DestinationOutOfBounds)?;
            insert_layer(cell, entity, layer)?;
        }
        if let Some(original_location) = self.location_component.insert(entity, location) {
            let original_cell = self.spatial_grid.get_checked_mut(original_location.coord);
            if let Some(original_layer) = original_location.layer {
                let should_match_entity = clear_layer(original_cell, original_layer);
                assert_eq!(
                    should_match_entity,
                    Some(entity),
                    "Current location of entity doesn't contain entity in spatial grid"
                );
            }
        }
        Ok(())
    }
    pub fn update_coord(&mut self, entity: Entity, coord: Coord) -> Result<(), UpdateError> {
        if let Some(location) = self.location_component.get_mut(entity) {
            if coord != location.coord {
                if let Some(layer) = location.layer {
                    let cell = self
                        .spatial_grid
                        .get_mut(coord)
                        .ok_or(UpdateError::DestinationOutOfBounds)?;
                    insert_layer(cell, entity, layer)?;
                    let original_cell = self.spatial_grid.get_checked_mut(location.coord);
                    let should_match_entity = clear_layer(original_cell, layer);
                    assert_eq!(
                        should_match_entity,
                        Some(entity),
                        "Current location of entity doesn't contain entity in spatial grid"
                    );
                }
                location.coord = coord;
            }
            Ok(())
        } else {
            self.update(entity, Location { coord, layer: None })
        }
    }
    pub fn remove(&mut self, entity: Entity) {
        if let Some(location) = self.location_component.remove(entity) {
            if let Some(layer) = location.layer {
                clear_layer(self.spatial_grid.get_checked_mut(location.coord), layer);
            }
        }
    }
    #[cfg(feature = "serialize")]
    fn to_serialize(&self) -> SpatialSerialize<L::Layer> {
        SpatialSerialize {
            entries: self.location_component.entries().clone(),
            size: self.spatial_grid.size(),
        }
    }
    #[cfg(feature = "serialize")]
    fn from_serialize(SpatialSerialize { entries, size }: SpatialSerialize<L::Layer>) -> Self {
        let location_component = entries.into_component_table();
        let mut spatial_grid: Grid<L> = Grid::new_default(size);
        for (entity, location) in location_component.iter() {
            if let Some(layer) = location.layer {
                let cell = spatial_grid.get_checked_mut(location.coord);
                let slot = cell.select_field_mut(layer);
                assert!(slot.is_none());
                *slot = Some(entity);
            }
        }
        Self {
            location_component,
            spatial_grid,
        }
    }
}

struct OccupiedBy(pub Entity);

impl From<OccupiedBy> for UpdateError {
    fn from(occupied_by: OccupiedBy) -> UpdateError {
        UpdateError::OccupiedBy(occupied_by.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    OccupiedBy(Entity),
    DestinationOutOfBounds,
}

impl UpdateError {
    pub fn unwrap_occupied_by(self) -> Entity {
        match self {
            Self::OccupiedBy(entity) => entity,
            _ => panic!("unexpected {:?} (expected OccupiedBy(_))", self),
        }
    }
}

fn insert_layer<L: Layers>(
    layers: &mut L,
    entity: Entity,
    layer: L::Layer,
) -> Result<(), OccupiedBy> {
    let layer_field = layers.select_field_mut(layer);
    if let Some(&occupant) = layer_field.as_ref() {
        Err(OccupiedBy(occupant))
    } else {
        *layer_field = Some(entity);
        Ok(())
    }
}
fn clear_layer<L: Layers>(layers: &mut L, layer: L::Layer) -> Option<Entity> {
    layers.select_field_mut(layer).take()
}

#[cfg(feature = "serialize")]
#[derive(Serialize, Deserialize)]
struct SpatialSerialize<L> {
    entries: ComponentTableEntries<Location<L>>,
    size: Size,
}

#[cfg(feature = "serialize")]
impl<L: Layers> Serialize for SpatialTable<L>
where
    L::Layer: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_serialize().serialize(s)
    }
}

#[cfg(feature = "serialize")]
impl<'a, L: Layers> Deserialize<'a> for SpatialTable<L>
where
    L::Layer: Deserialize<'a>,
{
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(Self::from_serialize)
    }
}

#[cfg(test)]
mod test {
    declare_layers_module! {
        layers {
            floor: Floor,
            feature: Feature,
            character: Character,
        }
    }
    use layers::{Layer, Layers};
    type SpatialTable = super::SpatialTable<Layers>;
    use super::{Coord, Location, Size, UpdateError};
    use entity_table::EntityAllocator;

    #[test]
    fn test() {
        let mut entity_allocator = EntityAllocator::default();
        let mut spatial_table = SpatialTable::new(Size::new(10, 10));
        let entity_a = entity_allocator.alloc();
        let entity_b = entity_allocator.alloc();
        let entity_c = entity_allocator.alloc();

        assert_eq!(spatial_table.location_of(entity_a), None);

        // try to place a feature out of bounds - should fail
        assert_eq!(
            spatial_table.update(
                entity_a,
                Location {
                    coord: Coord::new(-1, 10),
                    layer: Some(Layer::Feature),
                },
            ),
            Err(UpdateError::DestinationOutOfBounds),
        );

        // entity should not have been added
        assert_eq!(spatial_table.location_of(entity_a), None);

        // try to place a feature at valid coord
        assert_eq!(
            spatial_table.update(
                entity_a,
                Location {
                    coord: Coord::new(4, 2),
                    layer: Some(Layer::Feature),
                },
            ),
            Ok(()),
        );
        assert_eq!(
            spatial_table.location_of(entity_a).cloned(),
            Some(Location {
                coord: Coord::new(4, 2),
                layer: Some(Layer::Feature)
            })
        );

        // move feature to new coord
        assert_eq!(
            spatial_table.update_coord(entity_a, Coord::new(6, 7)),
            Ok(()),
        );
        assert_eq!(
            spatial_table.location_of(entity_a).cloned(),
            Some(Location {
                coord: Coord::new(6, 7),
                layer: Some(Layer::Feature)
            })
        );

        assert_eq!(spatial_table.location_of(entity_b), None);

        // try to add a new feature on top - should fail
        assert_eq!(
            spatial_table.update(
                entity_b,
                Location {
                    coord: Coord::new(6, 7),
                    layer: Some(Layer::Feature),
                },
            ),
            Err(UpdateError::OccupiedBy(entity_a)),
        );

        assert_eq!(spatial_table.location_of(entity_b), None);

        // add new feature in different coord
        assert_eq!(
            spatial_table.update(
                entity_b,
                Location {
                    coord: Coord::new(6, 8),
                    layer: Some(Layer::Feature),
                },
            ),
            Ok(()),
        );

        // try to move it to coord with existing feature
        assert_eq!(
            spatial_table.update_coord(entity_b, Coord::new(6, 7)),
            Err(UpdateError::OccupiedBy(entity_a)),
        );

        assert_eq!(spatial_table.coord_of(entity_b), Some(Coord::new(6, 8)));

        assert_eq!(spatial_table.location_of(entity_c), None);

        // add a character on top of an entity_a
        assert_eq!(
            spatial_table.update(
                entity_c,
                Location {
                    coord: Coord::new(6, 7),
                    layer: Some(Layer::Character),
                },
            ),
            Ok(()),
        );
        assert_eq!(spatial_table.coord_of(entity_c), Some(Coord::new(6, 7)));

        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: Some(entity_a),
                character: Some(entity_c),
                floor: None,
            },
        );
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 8)),
            Layers {
                feature: Some(entity_b),
                character: None,
                floor: None,
            },
        );

        spatial_table.remove(entity_a);
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: None,
                character: Some(entity_c),
                floor: None,
            },
        );
        assert_eq!(
            spatial_table.update_coord(entity_b, Coord::new(6, 7)),
            Ok(()),
        );
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: Some(entity_b),
                character: Some(entity_c),
                floor: None,
            },
        );
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 8)),
            Layers {
                feature: None,
                character: None,
                floor: None,
            },
        );
    }
}
