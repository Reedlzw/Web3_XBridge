use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";
// pub const DEPLOYER_KEY_STR: &str = "Tsfz1zyRMAJk4PKEwP13CzZxowuaEahRMXrsePrf6Ev";
pub const DEPLOYER_KEY_STR: &str = "Jk9fBdZBe83dsy5t8FWuk26LZhytWJCa7MXTqkiDEtF";
pub const TEST_MPC_STR: &str = "4JytEnivUZr9wVCsENs6dcWvzjYQF41RPkjQxwppo2j6";
pub const TEST_MPC_STR_2: &str = "5UYLAV5znKESoEoZT7orPGC5BmDtB5YsXwhFshhLqyeC";
pub const COMMISSION_RATE_LIMIT: u16 = 300;
pub const COMMISSION_DENOMINATOR: u64 = 10000;

pub mod dexrouter_program {
    crate::declare_id!("6m2CDdhRgxpH4WjvdzxAYbGxwdGUz5MziiL5jek2kBma");
    // declare_id!("4mmSpBByrSF4CeePgj7HT5aZkw1whbYCmjLw2xiMTZfg");
}
pub mod wrapped_sol {
    crate::declare_id!("So11111111111111111111111111111111111111112");
}
pub mod wormhole_core_program {
    crate::declare_id!("worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth");
}

pub mod wormhole_token_bridge_program {
    crate::declare_id!("wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb");
}

pub mod meson_program {
    crate::declare_id!("FR1SDyLUj7PrMbtkUCkDrBymk5eWrRmr3UvWFb5Kjbmd");
}

pub mod meson_contract_signer {
    crate:: declare_id!("8w2oGkk75arbcXyDyaBbFqMLXgpZpyKc9ikLZwP8qLNZ");
}

pub mod debridgedln_program {
    crate::declare_id!("src5qyZHqTqecJV4aY6Cb6zDZLMDzrDKKezs22MPHr4");
}

pub mod cctp_program {          
    crate::declare_id!("CCTPiPYPc6AsJuwueEnWgSgucamXDZwBd53dQ11YiKX3");
}

pub mod cctp_message_program {          
    crate::declare_id!("CCTPmbSD7gX1bxKPAmg77w8oFzNFpaQiQUWD43TKaecd");
}

pub mod allbridge_program {          
    use anchor_lang::declare_id;
    declare_id!("BrdgN2RPzEMWF96ZbnnJaUtQDQx7VRXYaHHbYCBvceWB");
}

pub mod allbridge_messager_program {          
    use anchor_lang::declare_id;
    declare_id!("AMsgYtqR3EXKfsz6Rj2cKnrYGwooaSk7BQGeyVBB5yjS");
}

pub mod allbridge_gas_program {          
    use anchor_lang::declare_id;
    declare_id!("GasB9dNMfXGXysMeTjQnnAnN38RmbrphtH5dkkWnMMvQ");
}

pub mod wanchain_program {
    crate::declare_id!("E3iKvJgGNycXrmsh2aryY25z29PpU4dvo4CBuXCKQiGB");
}

pub mod wanchain_sol_value {
    crate::declare_id!("AKXdNCG4GcTQ1knC7kno9bggHuq8MG9CCb8yQd8Nx2vL");
}

pub mod wanchain_fee_receiver {
    crate::declare_id!("CXxYYAtiUhdUagJNQ6UAB9gmHdxeujUPdn4iRg9HeuSz");
}

pub mod wanchain_admin_board_program {
    crate::declare_id!("7jYCM8k5Nvwg5vyPpLk2yjivQhexPDMXuK8CSbUKqL6B");
}

pub mod wanchain_config_account {
    crate::declare_id!("9o7zWu1n3q1MCAQp5y8RYmhhVjNpkfhpbSDMeYvjwhZP");
}

pub mod wanchain_circle_config_program {
    crate::declare_id!("dFYBRAFvZKq9F4mYGkLQu8DbfZRFrmi5dNSTDfwC3a8");
}

pub mod mayan_swift_program {
    crate::declare_id!("BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY");
}

pub mod mayan_fee_manager_program {
    crate::declare_id!("5VtQHnhs2pfVEr68qQsbTRwKh4JV5GTu9mBHgHFxpHeQ");
}

pub mod bridgers_program {
    crate::declare_id!("FDF8AxHB8UK7RS6xay6aBvwS3h7kez9gozqz14JyfKsg");
}

pub mod bridgers_dest_owner {
    crate::declare_id!("ZfctMHBkZNTqeYGE47ekxtydgXgpo9xKJCAasjaCLTU");
}

pub mod bridgers_vs_info {
    crate::declare_id!("2CtxEnat1bvq1KtKzZdze54aq9F8FhBECLqNPCNjVoFU");
}