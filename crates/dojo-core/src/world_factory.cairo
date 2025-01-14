use array::ArrayTrait;

#[starknet::contract]
mod WorldFactory {
    use array::ArrayTrait;
    use option::OptionTrait;
    use traits::Into;

    use starknet::{
        ClassHash, ContractAddress, contract_address::ContractAddressIntoFelt252,
        syscalls::deploy_syscall, get_caller_address
    };

    use dojo::interfaces::{IWorldDispatcher, IWorldDispatcherTrait};

    #[storage]
    struct Storage {
        world_class_hash: ClassHash,
        executor_address: ContractAddress,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        WorldSpawned: WorldSpawned
    }

    #[derive(Drop, starknet::Event)]
    struct WorldSpawned {
        address: ContractAddress
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        world_class_hash_: ClassHash,
        executor_address_: ContractAddress,
    ) {
        self.world_class_hash.write(world_class_hash_);
        self.executor_address.write(executor_address_);
    }

    #[external(v0)]
    fn spawn(
        ref self: ContractState, components: Array<ClassHash>, systems: Array<ClassHash>, 
    ) -> ContractAddress {
        // deploy world
        let mut world_constructor_calldata: Array<felt252> = ArrayTrait::new();
        world_constructor_calldata.append(self.executor_address.read().into());
        let world_class_hash = self.world_class_hash.read();
        let result = deploy_syscall(world_class_hash, 0, world_constructor_calldata.span(), true);
        let (world_address, _) = result.unwrap_syscall();
        let world = IWorldDispatcher { contract_address: world_address };

        // events
        self.emit(WorldSpawned { address: world_address });

        // register components
        let components_len = components.len();
        register_components(@self, components, components_len, 0, world_address);

        // register systems
        let systems_len = systems.len();
        register_systems(@self, systems, systems_len, 0, world_address);

        return world_address;
    }

    #[external(v0)]
    fn set_executor(ref self: ContractState, executor_address_: ContractAddress) {
        self.executor_address.write(executor_address_);
    }

    #[external(v0)]
    fn set_world(ref self: ContractState, class_hash: ClassHash) {
        self.world_class_hash.write(class_hash);
    }

    #[external(v0)]
    fn executor(self: @ContractState) -> ContractAddress {
        return self.executor_address.read();
    }

    #[external(v0)]
    fn world(self: @ContractState) -> ClassHash {
        return self.world_class_hash.read();
    }

    fn register_components(
        self: @ContractState,
        components: Array<ClassHash>,
        components_len: usize,
        index: usize,
        world_address: ContractAddress
    ) {
        if (index == components_len) {
            return ();
        }
        IWorldDispatcher {
            contract_address: world_address
        }.register_component(*components.at(index));
        return register_components(self, components, components_len, index + 1, world_address);
    }

    fn register_systems(
        self: @ContractState, systems: Array<ClassHash>, systems_len: usize, index: usize, world_address: ContractAddress
    ) {
        if (index == systems_len) {
            return ();
        }
        IWorldDispatcher { contract_address: world_address }.register_system(*systems.at(index));
        return register_systems(self, systems, systems_len, index + 1, world_address);
    }
}
