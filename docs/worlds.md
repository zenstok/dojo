# Worlds

The worlds interface is as follows:

```rust
trait World {
    #[event]
    fn StoreSetRecord(table_id: felt252, key: Span<felt252>, value: Span<felt252>) {}

    #[event]
    fn StoreSetField(table_id: felt252, key: Span<felt252>, offset: u8, value: Span<felt252>) {}

    #[event]
    fn ComponentRegistered(name: felt252, class_hash: ClassHash) {}

    #[event]
    fn SystemRegistered(name: felt252, class_hash: ClassHash) {}

    // Returns a globally unique identifier.
    #[external(v0)]
    fn uuid() -> felt252;

    // Returns a globally unique identifier.
    #[external(v0)]
    fn get(component: felt252, query: Query, offset: u8, length: usize) -> Span<felt252>;

    // Returns all entities that contain the component.
    #[external(v0)]
    fn entities(component: felt252, partition: felt252) -> Array<Query>;

    // Sets a components value.
    #[external(v0)]
    fn set(component: felt252, query: Query, offset: u8, value: Span<felt252>);

    // Returns all entities that contain the component.
    #[external(v0)]
    fn register_component(name: felt252, class_hash: ClassHash);

    // Returns all entities that contain the component.
    #[external(v0)]
    fn register_system(name: felt252, class_hash: ClassHash);
}
```
