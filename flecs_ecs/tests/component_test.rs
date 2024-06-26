use flecs_ecs::core::{entity::Entity, world::World};

mod common;
use common::*;

#[test]
fn temp_test_hook() {
    static mut COUNT: u32 = 0;
    static mut COUNT2: u32 = 0;
    {
        let world = World::new();
        world
            .component::<Position>()
            .on_add(|_e: Entity, p: &mut Position| {
                unsafe { COUNT += 1 };
                p.x = 10;
                p.y = 20;
            })
            .on_remove(|_e: Entity, _p: &mut Position| {
                unsafe { COUNT -= 1 };
            });

        world
            .component::<Velocity>()
            .on_set(|_e: Entity, v: &mut Velocity| {
                unsafe { COUNT2 += 1 };
                v.x *= 10;
                v.y *= 10;
            });

        assert_eq!(unsafe { COUNT }, 0);

        let entity = world.new_entity().add::<Position>();

        assert_eq!(unsafe { COUNT }, 1);
        let pos_e1 = entity.get::<Position>().unwrap();
        assert_eq!(pos_e1.x, 10);
        assert_eq!(pos_e1.y, 20);

        entity.add::<Position>();
        assert_eq!(unsafe { COUNT }, 1);
        let pos_e1 = entity.get::<Position>().unwrap();
        assert_eq!(pos_e1.x, 10);
        assert_eq!(pos_e1.y, 20);

        let entity2 = world.new_entity().add::<Position>();
        assert_eq!(unsafe { COUNT }, 2);
        let pos_e2 = entity2.get::<Position>().unwrap();
        assert_eq!(pos_e1.x, 10);
        assert_eq!(pos_e1.y, 20);
        assert_eq!(pos_e2.x, 10);
        assert_eq!(pos_e2.y, 20);

        entity.add::<Velocity>();
        assert_eq!(unsafe { COUNT2 }, 0);
        let vel_e1 = entity.get::<Velocity>().unwrap();
        // dangerous uninitialized values
        assert_ne!(vel_e1.x, 0);
        assert_ne!(vel_e1.y, 0);
        entity.remove::<Velocity>();

        entity.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 1);

        entity2.set(Velocity { x: 3, y: 5 });
        let vel_e2 = entity2.get::<Velocity>().unwrap();
        assert_eq!(vel_e2.x, 30);
        assert_eq!(vel_e2.y, 50);
        assert_eq!(unsafe { COUNT2 }, 1);

        entity.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 1);
        entity2.remove::<Position>();
        assert_eq!(unsafe { COUNT }, 0);
    }
}
