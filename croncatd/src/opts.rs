//!
//! `croncatd` CLI option builder.
//!

use enum_display::EnumDisplay;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "croncatd", about = "The croncat agent daemon.")]
pub struct Opts {
    /// Debug mode
    #[structopt(short, long)]
    pub debug: bool,

    /// Whether to print nice little things like the banner and a goodbye
    #[structopt(long)]
    pub no_frills: bool,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
}

#[derive(Debug, StructOpt, Clone, EnumDisplay)]
#[enum_display(case = "Kebab")]
pub enum Command {
    /// Registers an agent, placing them in the pending queue unless it's the first agent.
    RegisterAgent {
        payable_account_id: Option<String>,

        #[structopt(long, default_value = "agent")]
        sender_name: String,
        #[structopt(long, default_value = "local")]
        chain_id: String,
    },

    /// Get the agent's supported bech32 accounts
    GetAgentAccounts {
        #[structopt(long, default_value = "agent")]
        sender_name: String,
        #[structopt(long, default_value = "local")]
        chain_id: String,
    },

    /// Get the agent's status (pending/active)
    GetAgentStatus {
        account_id: String,
        #[structopt(long, default_value = "local")]
        chain_id: String,
    },

    /// Get the agent's tasks they're assigned to fulfill
    GetAgentTasks {
        account_addr: String,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Unregisters the agent from being in the queue with other agents
    UnregisterAgent {
        #[structopt(long, default_value = "agent")]
        sender_name: String,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Update the agent's configuration
    UpdateAgent {
        payable_account_id: String,
        #[structopt(long, default_value = "agent")]
        sender_name: String,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Withdraw the agent's funds to the payable account ID
    Withdraw {
        #[structopt(long, default_value = "agent")]
        sender_name: String,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Get contract's state
    #[cfg(feature = "debug")]
    GetState {
        from_index: Option<u64>,
        limit: Option<u64>,
    },

    /// Show all task(s) information
    Tasks {
        from_index: Option<u64>,
        limit: Option<u64>,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Starts the Croncat agent, allowing it to fulfill tasks
    Go {
        #[structopt(long, default_value = "agent")]
        sender_name: String,
        /// Allow agent to do tasks with rules, uses more computer resources
        #[structopt(long, short = "r")]
        with_rules: bool,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Gets the configuration from the Croncat manager contract
    Info {
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Generates a new keypair and agent account (good first step)
    GenerateMnemonic {
        #[structopt(long, default_value = "agent")]
        new_name: String,
        /// Recover agent from mnemonic phrase. Please do not use your own account!
        #[structopt(long)]
        mnemonic: Option<String>,
    },

    /// (in progress) Send native tokens to an address
    DepositUjunox {
        account_id: String,
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
    },

    /// Sensitive. Shows all details about agents on this machine
    GetAgent {
        #[structopt(long, default_value = "agent")]
        name: String,
    },

    /// Setup an agent as a system service (systemd)
    SetupService {
        #[structopt(long, short, default_value = "local")]
        chain_id: String,
        #[structopt(long)]
        output: Option<String>,
    },
}
