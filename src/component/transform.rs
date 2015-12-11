use std::collections::{HashMap, HashSet};
use std::cell::{Cell, RefCell, Ref, RefMut};

use math::*;
use stopwatch::Stopwatch;

use ecs::{Entity, System, ComponentManager};
use scene::Scene;
use super::{EntityMap, EntitySet};

#[derive(Debug, Clone)]
pub struct TransformManager {
    transforms: Vec<Vec<RefCell<Transform>>>,
    entities: Vec<Vec<(Entity, Option<Entity>)>>,

    /// A map between the entity owning the transform and the location of the transform.
    ///
    /// The first value of the mapped tuple is the row containing the transform, the
    /// second is the index of the transform within that row.
    indices: EntityMap<(usize, usize)>,

    marked_for_destroy: RefCell<EntitySet>,
}

impl TransformManager {
    pub fn new() -> TransformManager {
        let mut transform_manager = TransformManager {
            transforms: Vec::new(),
            entities: Vec::new(),
            indices: HashMap::default(),
            marked_for_destroy: RefCell::new(HashSet::default()),
        };

        transform_manager.transforms.push(Vec::new());
        transform_manager.entities.push(Vec::new());
        transform_manager
    }

    pub fn assign(&mut self, entity: Entity) -> RefMut<Transform> {
        let index = self.transforms[0].len();
        self.transforms[0].push(RefCell::new(Transform::new()));
        self.entities[0].push((entity, None));

        assert!(self.transforms[0].len() == self.entities[0].len());

        self.indices.insert(entity, (0, index));
        self.transforms[0][index].borrow_mut()
    }

    pub fn get(&self, entity: Entity) -> Ref<Transform> {
        let (row, index) = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        self.transforms[row][index].borrow()
    }

    pub fn get_mut(&self, entity: Entity) -> RefMut<Transform> {
        let (row, index) = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        self.transforms[row][index].borrow_mut()
    }

    pub fn set_child(&mut self, parent: Entity, child: Entity) {
        // Get the indices of the parent.
        let (parent_row, _) = *self.indices.get(&parent).unwrap();
        let child_row = parent_row + 1;

        // Move the child and all of its children to the correct row.
        self.set_row_recursive(child, Some(parent), child_row);
    }

    fn set_row_recursive(&mut self, entity: Entity, parent: Option<Entity>, new_row: usize) {
        debug_assert!((new_row == 0 && parent.is_none()) || (new_row > 0 && parent.is_some()));

        // Remove old transform component.
        let (old_row, _) = *self.indices.get(&entity).unwrap(); // TODO: Don't panic? If this fails an invariant somewhere else was broken.
        let transform = self.remove(entity);

        // Ensure that there are enough rows for the child.
        while self.transforms.len() < new_row + 1 {
            self.transforms.push(Vec::new());
            self.entities.push(Vec::new());
        }

        // Add the child to the correct row.
        let child_index = self.transforms[new_row].len();
        self.transforms[new_row].push(RefCell::new(transform));
        self.entities[new_row].push((entity, parent));

        // Update the index map.
        self.indices.insert(entity, (new_row, child_index));

        // Update all children.
        // TODO: We shouldn't have to clone the list here, but Rust's ownership rules mean that we
        // can't compile if we don't (which is completely valid in this case). Once we implement a
        // more stable form of storage for transform nodes (where pointers to nodes are stable)
        // then cloning should be able to go away.
        for (child, maybe_parent) in self.entities[old_row + 1].clone() {
            match maybe_parent {
                Some(parent) if parent == entity => {
                    self.set_row_recursive(child, Some(entity), new_row + 1);
                },
                _ => {},
            }
        }
    }

    pub fn update_single(&self, entity: Entity) {
        let transform = self.get(entity);

        let (row, index) = *self.indices.get(&entity).expect("Transform manager does not contain a transform for the given entity.");
        let (_, parent) = self.entities[row][index];
        match parent {
            None => {
                DUMMY_TRANSFORM.with(|parent| {
                    transform.update(parent);
                })
            },
            Some(parent) => {
                // First update parent.
                self.update_single(parent);

                // Now update self with the parent's updated transform.
                let parent_transform = self.get(parent);
                transform.update(&*parent_transform);
            }
        }
    }

    /// Walks the transform hierarchy depth-first, invoking `callback` with each entity and its transform.
    ///
    /// # Details
    ///
    /// The callback is also invoked for the root entity. If the root entity does not have a transform
    /// the callback is never invoked.
    pub fn walk_hierarchy<F: FnMut(Entity, &mut Transform)>(&self, entity: Entity, callback: &mut F) {
        if let Some(&(row, index)) = self.indices.get(&entity) {
            let mut transform = self.transforms[row][index].borrow_mut();
            callback(entity, &mut *transform);

            let child_row = row + 1;
            if self.transforms.len() > child_row {
                for (child_index, _) in self.entities[child_row].iter().enumerate().filter(|&(_, &(_, parent))| parent.unwrap() == entity) {
                    let (child_entity, _) = self.entities[child_row][child_index];
                    self.walk_hierarchy(child_entity, callback);
                }
            }
        }
    }

    /// Walks the transform hierarchy depth-first, invoking `callback` with each entity.
    ///
    /// # Details
    ///
    /// The callback is also invoked for the root entity. If the root entity does not have a transform
    /// the callback is never invoked. Note that the transform itself is not passed to the callback,
    /// if you need to access the transform use `walk_hierarchy()` instead.
    pub fn walk_children<F: FnMut(Entity)>(&self, entity: Entity, callback: &mut F) {
        if let Some(&(row, _)) = self.indices.get(&entity) {
            callback(entity);

            let child_row = row + 1;
            if self.transforms.len() > child_row {
                for (child_index, _) in self.entities[child_row].iter().enumerate().filter(|&(_, &(_, parent))| parent.unwrap() == entity) {
                    let (child_entity, _) = self.entities[child_row][child_index];
                    self.walk_children(child_entity, callback);
                }
            }
        }
    }

    /// Marks the transform associated with the entity for destruction.
    ///
    /// # Details
    ///
    /// Components marked for destruction are destroyed at the end of every frame. It can be used
    /// to destroy components without needing a mutable borrow on the component manager.
    ///
    /// TODO: Actually support deferred destruction.
    pub fn destroy(&self, entity: Entity) {
        let mut marked_for_destroy = self.marked_for_destroy.borrow_mut();
        marked_for_destroy.insert(entity); // TODO: Warning, error if entity has already been marked?
    }

    pub fn destroy_immediate(&mut self, entity: Entity) {
        self.remove(entity);
    }

    // Removes and returns the transform associated with the given entity.
    //
    // # Details
    //
    // NOTE: This does not handle updating/removing children. So be warned.
    fn remove(&mut self, entity: Entity) -> Transform {
        // Retrieve indices of removed entity and the one it's swapped with.
        let (row, index) = self.indices.remove(&entity).unwrap();
        debug_assert!(self.transforms[row].len() == self.entities[row].len());

        // Remove transform and the associate entity.
        let (removed_entity, _) = self.entities[row].swap_remove(index);
        debug_assert!(removed_entity == entity);

        // Update the index mapping for the moved entity, but only if the one we removed
        // wasn't the only one in the row (or the last one in the row).
        if index != self.entities[row].len() {
            let (moved_entity, _) = self.entities[row][index];
            self.indices.insert(moved_entity, (row, index));
        }

        // Defer removing the transform until the very end to avoid a bunch of memcpys.
        // Transform is a pretty fat struct so if we remove it, cache it to a variable,
        // and then return it at the end we wind up with 2 or 3 memcpys. Doing it all at
        // once at the end (hopefully) means only a single memcpy.
        self.transforms[row].swap_remove(index).into_inner()
    }
}

impl ComponentManager for TransformManager {
    fn destroy_all(&self, entity: Entity) {
        self.marked_for_destroy.borrow_mut().insert(entity);
    }

    fn destroy_marked(&mut self) {
        let mut marked_for_destroy = RefCell::new(HashSet::default());
        ::std::mem::swap(&mut marked_for_destroy, &mut self.marked_for_destroy);
        let mut marked_for_destroy = marked_for_destroy.into_inner();
        for entity in marked_for_destroy.drain() {
            self.destroy_immediate(entity);
        }
    }
}

thread_local!(static DUMMY_TRANSFORM: Transform = Transform::new());

/// TODO: This should be module-level documentation.
///
/// A component representing the total transform (position, orientation,
/// and scale) of an object in the world.
///
/// # Details
///
/// The `Transform` component is a fundamental part of the Gunship engine.
/// It has the dual role of managing each individual entity's local transformation,
/// as well as representing the individual nodes within the scene hierarchy.
///
/// ## Scene hierarchy
///
/// Each transform component may have one parent and any number of children. If a transform has
/// a parent then its world transformation is the concatenation of its local transformation with
/// its parent's world transformation. Using simple combinations of nested transforms can allow
/// otherwise complex patterns of movement and positioning to be much easier to represent.
///
/// Transforms that have no parent are said to be at the root level and have the property
/// that their local transformation is also their world transformation. If a transform is
/// known to be at the root of the hierarchy it is recommended that its local values be modified
/// directly to achieve best performance.
#[derive(Debug, Clone)]
pub struct Transform {
    position:         Point,
    rotation:         Quaternion,
    scale:            Vector3,
    local_matrix:     Cell<Matrix4>,
    position_derived: Cell<Point>,
    rotation_derived: Cell<Quaternion>,
    scale_derived:    Cell<Vector3>,
    matrix_derived:   Cell<Matrix4>,
    out_of_date:      Cell<bool>,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position:         Point::origin(),
            rotation:         Quaternion::identity(),
            scale:            Vector3::one(),
            local_matrix:     Cell::new(Matrix4::identity()),
            position_derived: Cell::new(Point::origin()),
            rotation_derived: Cell::new(Quaternion::identity()),
            scale_derived:    Cell::new(Vector3::one()),
            matrix_derived:   Cell::new(Matrix4::identity()),
            out_of_date:      Cell::new(false),
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn set_position(&mut self, new_position: Point) {
        self.position = new_position;
        self.out_of_date.set(true);
    }

    pub fn rotation(&self) -> Quaternion {
        self.rotation
    }

    pub fn set_rotation(&mut self, new_rotation: Quaternion) {
        self.rotation = new_rotation;
        self.out_of_date.set(true);
    }

    pub fn scale(&self) -> Vector3 {
        self.scale
    }

    pub fn set_scale(&mut self, new_scale: Vector3) {
        self.scale = new_scale;
        self.out_of_date.set(true);
    }

    /// Retrieves the derived position of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn position_derived(&self) -> Point {
        assert!(!self.out_of_date.get());

        self.position_derived.get()
    }

    /// Retrieves the derived rotation of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn rotation_derived(&self) -> Quaternion {
        assert!(!self.out_of_date.get());

        self.rotation_derived.get()
    }

    /// Retrieves the derived scale of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn scale_derived(&self) -> Vector3 {
        assert!(!self.out_of_date.get());

        self.scale_derived.get()
    }

    /// Retrieves the composite matrix representing the local transform.
    ///
    /// # Details
    ///
    /// The composite matrix combines the affine matrices representing translation,
    /// scale, and rotation into a single transformation matrix. The local maxtrix does
    /// not include the parent's transformation. The local matrix transforms a local point
    /// into the parent's coordinate system.
    pub fn local_matrix(&self) -> Matrix4 {
        if self.out_of_date.get() {
            let local_matrix =
                Matrix4::from_point(self.position)
                * (self.rotation.as_matrix4() * Matrix4::from_scale_vector(self.scale));
            self.local_matrix.set(local_matrix);
        }

        self.local_matrix.get()
    }

    pub fn derived_matrix(&self) -> Matrix4 {
        assert!(!self.out_of_date.get());

        self.matrix_derived.get()
    }

    pub fn derived_normal_matrix(&self) -> Matrix4 {
        assert!(!self.out_of_date.get());

        let inverse =
            Matrix4::from_scale_vector(1.0 / self.scale_derived.get())
          * (self.rotation_derived.get().as_matrix4().transpose()
          *  Matrix4::from_point(-self.position_derived.get()));

        inverse.transpose()
    }

    pub fn translate(&mut self, translation: Vector3) {
        self.position = self.position + translation;
        self.out_of_date.set(true);
    }

    pub fn rotate(&mut self, rotation: Quaternion) {
        self.rotation = self.rotation * rotation;
        self.out_of_date.set(true);
    }

    pub fn look_at(&mut self, interest: Point, up: Vector3) {
        let forward = interest - self.position;
        self.rotation = Quaternion::look_rotation(forward, up);
        self.out_of_date.set(true);
    }

    pub fn look_direction(&mut self, forward: Vector3, up: Vector3) {
        self.rotation = Quaternion::look_rotation(forward, up);
        self.out_of_date.set(true);
    }

    pub fn forward(&self) -> Vector3 {
        let matrix = Matrix3::from_quaternion(self.rotation);
        -matrix.z_part()
    }

    pub fn right(&self) -> Vector3 {
        let matrix = Matrix3::from_quaternion(self.rotation);
        matrix.x_part()
    }

    pub fn up(&self) -> Vector3 {
        let matrix = Matrix3::from_quaternion(self.rotation);
        matrix.y_part()
    }

    /// Updates the local and derived matrices for the transform.
    fn update(&self, parent: &Transform) {
        let local_matrix = self.local_matrix();

        let derived_matrix = parent.derived_matrix() * local_matrix;
        self.matrix_derived.set(derived_matrix);

        self.position_derived.set(derived_matrix.translation_part());
        self.rotation_derived.set(parent.rotation_derived() * self.rotation);
        self.scale_derived.set(self.scale * parent.scale_derived());

        self.out_of_date.set(false);
    }
}

pub fn transform_update(scene: &Scene, _: f32) {
    let _stopwatch = Stopwatch::new("transform update");

    let transform_manager = scene.get_manager::<TransformManager>();

    for (transform_row, entity_row) in transform_manager.transforms.iter().zip(transform_manager.entities.iter()) {
        for (transform, &(_, parent)) in transform_row.iter().zip(entity_row.iter()) {
            // Retrieve the parent's transformation matrix, using the identity
            // matrix if the transform has no parent.
            match parent {
                None => {
                    DUMMY_TRANSFORM.with(|parent| {
                        transform.borrow().update(parent);
                    });
                },
                Some(parent) => {
                    let parent_transform = transform_manager.get(parent);
                    transform.borrow().update(&*parent_transform);
                }
            };
        }
    }
}
