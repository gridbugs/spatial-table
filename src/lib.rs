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
    type Layer: Copy + PartialEq + Eq;
    fn select_field_mut(&mut self, layer: Self::Layer) -> &mut Option<Entity>;
}

#[cfg(not(feature = "serialize"))]
#[macro_export]
macro_rules! declare_layers_module {
    { $module_name:ident { $($field_name:ident: $variant_name:ident,)* } } => {
        mod $module_name {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct LayerTable<T> {
                $(pub $field_name: T,)*
            }

            pub type Layers = LayerTable<Option<$crate::Entity>>;

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Layer {
                $($variant_name,)*
            }

            impl<T> Default for LayerTable<Option<T>> {
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

            impl<T> LayerTable<T> {
                #[allow(unused)]
                pub fn map<U, F: FnMut(&T) -> U>(&self, mut f: F) -> LayerTable<U> {
                    LayerTable {
                        $($field_name: f(&self.$field_name),)*
                    }
                }

                #[allow(unused)]
                pub fn for_each<F: FnMut(&T)>(&self, mut f: F) {
                    $(f(&self.$field_name);)*
                }

                #[allow(unused)]
                pub fn for_each_enumerate<F: FnMut(&T, Layer)>(&self, mut f: F) {
                    $(f(&self.$field_name, Layer::$variant_name);)*
                }
            }

            impl<T> LayerTable<Option<T>> {
                #[allow(unused)]
                pub fn option_map<U, F: FnMut(&T) -> U>(&self, mut f: F) -> LayerTable<Option<U>> {
                    self.map(|ot| ot.as_ref().map(|t| f(t)))
                }

                #[allow(unused)]
                pub fn option_and_then<U, F: FnMut(&T) -> Option<U>>(&self, mut f: F) -> LayerTable<Option<U>> {
                    self.map(|ot| ot.as_ref().and_then(|t| f(t)))
                }

                #[allow(unused)]
                pub fn option_for_each<F: FnMut(&T)>(&self, mut f: F) {
                    $(if let Some(t) = self.$field_name.as_ref() { f(t); })*
                }

                #[allow(unused)]
                pub fn option_for_each_enumerate<F: FnMut(&T, Layer)>(&self, mut f: F) {
                    $(if let Some(t) = self.$field_name.as_ref() { f(t, Layer::$variant_name); })*
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
            pub struct LayerTable<T> {
                $(pub $field_name: T,)*
            }

            pub type Layers = LayerTable<Option<$crate::Entity>>;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, $crate::serde::Serialize, $crate::serde::Deserialize)]
            pub enum Layer {
                $($variant_name,)*
            }

            impl<T> Default for LayerTable<Option<T>> {
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

            impl<T> LayerTable<T> {
                #[allow(unused)]
                pub fn map<U, F: FnMut(&T) -> U>(&self, mut f: F) -> LayerTable<U> {
                    LayerTable {
                        $($field_name: f(&self.$field_name),)*
                    }
                }

                #[allow(unused)]
                pub fn for_each<F: FnMut(&T)>(&self, mut f: F) {
                    $(f(&self.$field_name);)*
                }

                #[allow(unused)]
                pub fn for_each_enumerate<F: FnMut(&T, Layer)>(&self, mut f: F) {
                    $(f(&self.$field_name, Layer::$variant_name);)*
                }
            }

            impl<T> LayerTable<Option<T>> {
                #[allow(unused)]
                pub fn option_map<U, F: FnMut(&T) -> U>(&self, mut f: F) -> LayerTable<Option<U>> {
                    self.map(|ot| ot.as_ref().map(|t| f(t)))
                }

                #[allow(unused)]
                pub fn option_and_then<U, F: FnMut(&T) -> Option<U>>(&self, mut f: F) -> LayerTable<Option<U>> {
                    self.map(|ot| ot.as_ref().and_then(|t| f(t)))
                }

                #[allow(unused)]
                pub fn option_for_each<F: FnMut(&T)>(&self, mut f: F) {
                    $(if let Some(t) = self.$field_name.as_ref() { f(t); })*
                }

                #[allow(unused)]
                pub fn option_for_each_enumerate<F: FnMut(&T, Layer)>(&self, mut f: F) {
                    $(if let Some(t) = self.$field_name.as_ref() { f(t, Layer::$variant_name); })*
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

impl<L> From<(Coord, L)> for Location<L> {
    fn from((coord, layer): (Coord, L)) -> Self {
        Self {
            coord,
            layer: Some(layer),
        }
    }
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
    pub fn clear(&mut self) {
        self.location_component.clear();
        for cell in self.spatial_grid.iter_mut() {
            *cell = Default::default();
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
    pub fn layer_of(&self, entity: Entity) -> Option<L::Layer> {
        self.location_of(entity).and_then(|l| l.layer)
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
                debug_assert_eq!(
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
                    debug_assert_eq!(
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
    pub fn update_layer(
        &mut self,
        entity: Entity,
        layer: L::Layer,
    ) -> Result<(), UpdateLayerError> {
        if let Some(location) = self.location_component.get_mut(entity) {
            if Some(layer) != location.layer {
                debug_assert!(
                    location.coord.is_valid(self.spatial_grid.size()),
                    "Current location is outside the bounds of spatial grid"
                );
                let cell = self.spatial_grid.get_mut(location.coord).unwrap();
                let dest_entity_slot = cell.select_field_mut(layer);
                if let Some(dest_entity) = dest_entity_slot {
                    return Err(UpdateLayerError::OccupiedBy(*dest_entity));
                }
                *dest_entity_slot = Some(entity);
                if let Some(current_layer) = location.layer {
                    let source_entity_slot = cell.select_field_mut(current_layer);
                    debug_assert_eq!(*source_entity_slot, Some(entity));
                    *source_entity_slot = None;
                }
                location.layer = Some(layer);
            }
            Ok(())
        } else {
            Err(UpdateLayerError::EntityHasNoCoord)
        }
    }
    pub fn clear_layer(&mut self, entity: Entity) -> Result<(), EntityHasNoCoord> {
        if let Some(location) = self.location_component.get_mut(entity) {
            if let Some(layer) = location.layer {
                debug_assert!(
                    location.coord.is_valid(self.spatial_grid.size()),
                    "Current location is outside the bounds of spatial grid"
                );
                let cell = self.spatial_grid.get_mut(location.coord).unwrap();
                let source_entity_slot = cell.select_field_mut(layer);
                debug_assert_eq!(*source_entity_slot, Some(entity));
                *source_entity_slot = None;
                location.layer = None;
            }
            Ok(())
        } else {
            Err(EntityHasNoCoord)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateLayerError {
    OccupiedBy(Entity),
    EntityHasNoCoord,
}

impl UpdateLayerError {
    pub fn unwrap_occupied_by(self) -> Entity {
        match self {
            Self::OccupiedBy(entity) => entity,
            _ => panic!("unexpected {:?} (expected OccupiedBy(_))", self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityHasNoCoord;

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
            feature: Feature,
            character: Character,
        }
    }
    use layers::{Layer, Layers};
    type SpatialTable = super::SpatialTable<Layers>;
    use super::{Coord, Location, Size, UpdateError, UpdateLayerError};
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
            },
        );
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 8)),
            Layers {
                feature: Some(entity_b),
                character: None,
            },
        );

        spatial_table
            .update_layer(entity_a, Layer::Feature)
            .unwrap();
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: Some(entity_a),
                character: Some(entity_c),
            },
        );

        assert_eq!(
            spatial_table.update_layer(entity_a, Layer::Character),
            Err(UpdateLayerError::OccupiedBy(entity_c))
        );

        spatial_table
            .update_layer(entity_b, Layer::Character)
            .unwrap();
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 8)),
            Layers {
                feature: None,
                character: Some(entity_b),
            },
        );
        assert_eq!(spatial_table.layer_of(entity_b), Some(Layer::Character));
        spatial_table
            .update_layer(entity_b, Layer::Feature)
            .unwrap();
        assert_eq!(spatial_table.layer_of(entity_b), Some(Layer::Feature));

        spatial_table.remove(entity_a);
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: None,
                character: Some(entity_c),
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
            },
        );
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 8)),
            Layers {
                feature: None,
                character: None,
            },
        );

        spatial_table.clear_layer(entity_b).unwrap();
        assert_eq!(
            *spatial_table.layers_at_checked(Coord::new(6, 7)),
            Layers {
                feature: None,
                character: Some(entity_c),
            },
        );
        assert_eq!(spatial_table.coord_of(entity_b), Some(Coord::new(6, 7)));
        assert_eq!(spatial_table.layer_of(entity_b), None);
    }
}
