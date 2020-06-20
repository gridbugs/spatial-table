use entity_table::{ComponentTable, ComponentTableEntries, Entity};
use grid_2d::{Coord, Grid, Size};
use serde::{Deserialize, Serialize};

pub trait Layers: Default {
    type Layer: Copy;
    fn select_field_mut(&mut self, layer: Self::Layer) -> &mut Option<Entity>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location<L> {
    pub coord: Coord,
    pub layer: Option<L>,
}

#[derive(Debug)]
pub struct SpatialTable<L: Layers> {
    location_component: ComponentTable<Location<L::Layer>>,
    spatial_grid: Grid<L>,
}

impl<L: Layers> SpatialTable<L> {
    pub fn new(size: Size) -> Self {
        let location_component = ComponentTable::default();
        let spatial_grid = Grid::new_default(size);
        Self {
            location_component,
            spatial_grid,
        }
    }
    pub fn enumerate(&self) -> impl Iterator<Item = (Coord, &L)> {
        self.spatial_grid.enumerate()
    }
    pub fn grid_size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn get_cell(&self, coord: Coord) -> Option<&L> {
        self.spatial_grid.get(coord)
    }
    pub fn get_cell_checked(&self, coord: Coord) -> &L {
        self.spatial_grid.get_checked(coord)
    }
    pub fn location(&self, entity: Entity) -> Option<&Location<L::Layer>> {
        self.location_component.get(entity)
    }
    pub fn coord(&self, entity: Entity) -> Option<Coord> {
        self.location(entity).map(|l| l.coord)
    }
    pub fn insert(
        &mut self,
        entity: Entity,
        location: Location<L::Layer>,
    ) -> Result<(), OccupiedBy> {
        if let Some(layer) = location.layer {
            let cell = self.spatial_grid.get_checked_mut(location.coord);
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
    pub fn update_coord(&mut self, entity: Entity, coord: Coord) -> Result<(), OccupiedBy> {
        if let Some(location) = self.location_component.get_mut(entity) {
            if coord != location.coord {
                if let Some(layer) = location.layer {
                    insert_layer(self.spatial_grid.get_checked_mut(coord), entity, layer)?;
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
            self.insert(entity, Location { coord, layer: None })
        }
    }
    pub fn remove(&mut self, entity: Entity) {
        if let Some(location) = self.location_component.remove(entity) {
            if let Some(layer) = location.layer {
                clear_layer(self.spatial_grid.get_checked_mut(location.coord), layer);
            }
        }
    }
    fn to_serialize(&self) -> SpatialSerialize<L::Layer> {
        SpatialSerialize {
            entries: self.location_component.entries().clone(),
            size: self.spatial_grid.size(),
        }
    }
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

#[derive(Debug)]
pub struct OccupiedBy(pub Entity);

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

#[derive(Serialize, Deserialize)]
struct SpatialSerialize<L> {
    entries: ComponentTableEntries<Location<L>>,
    size: Size,
}

impl<L: Layers> Serialize for SpatialTable<L>
where
    L::Layer: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_serialize().serialize(s)
    }
}

impl<'a, L: Layers> Deserialize<'a> for SpatialTable<L>
where
    L::Layer: Deserialize<'a>,
{
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(Self::from_serialize)
    }
}
