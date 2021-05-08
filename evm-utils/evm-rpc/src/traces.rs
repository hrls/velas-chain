use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

use primitive_types::{H160 as Address, H256, U256};
use std::collections::BTreeMap;

pub use jsonrpc_core::Result as RpcResult;

pub type BlockNumber = u64;
pub type Bytes = Vec<u8>;

#[rpc]
pub trait TracesERPC {
    type Metadata;

    #[rpc(name = "trace_block")]
    fn block_traces(&self, block: BlockNumber) -> RpcResult<Option<Vec<Trace>>>;

    #[rpc(name = "trace_replayBlockTransactions")]
    fn replay_block_transactions(
        &self,
        block: BlockNumber,
        options: TraceOptions,
    ) -> RpcResult<Vec<TraceResultsWithTransactionHash>>;

    // trace_replayTransaction
    // fn transaction_traces(&self, _: H256) -> RpcResult<Option<Vec<Trace>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceOptions {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// TODO: sync ser/de w/ LocalizedTrace
pub struct Trace {
    pub action: Action,
    pub result: Res,
    pub subtraces: usize,

    pub trace_address: Vec<usize>,
    pub transaction_number: Option<usize>,
    pub transaction_hash: Option<H256>,
    pub block_number: BlockNumber,
    pub block_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceResultsWithTransactionHash {
    pub output: Bytes,
    pub trace: Vec<Trace>,
    pub vm_trace: Option<vm::Trace>,
    pub state_diff: Option<StateDiff>,
    pub transaction_hash: H256,
}

pub mod vm {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Trace {
        pub code: Bytes,
        pub ops: Vec<Operation>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Operation {
        pub pc: usize,
        pub cost: U256,
        pub ex: Option<ExecutedOperation>,
        pub sub: Option<Trace>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ExecutedOperation {
        pub used: u64,
        pub push: Vec<U256>,
        pub mem: Option<MemoryDiff>,
        pub store: Option<StorageDiff>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryDiff {
        pub offset: usize,
        pub data: Bytes,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageDiff {
        pub key: U256,
        pub val: U256,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff(BTreeMap<Address, AccountDiff>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDiff {
    pub balance: Diff<U256>,
    pub nonce: Diff<U256>,
    pub code: Diff<Bytes>,
    pub storage: BTreeMap<H256, Diff<H256>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Diff<T> {
    #[serde(rename = "=")]
    Same,
    #[serde(rename = "+")]
    Born(T),
    #[serde(rename = "-")]
    Died(T),
    #[serde(rename = "*")]
    Changed(ChangedType<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ChangedType<T> {
    from: T,
    to: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Call {
        from: Address,
        to: Address,
        value: U256,
        gas: U256,
        input: Bytes,
        call_type: Option<CallType>, // NOTE: parity has some specifics
    },
    Create {
        from: Address,
        value: U256,
        gas: U256,
        init: Bytes,
        creation_method: CreationMethod,
    },
    Suicide {
        address: Address,
        refund_address: Address,
        balance: U256,
    },
    Reward {
        author: Address,
        value: U256,
        reward_type: RewardType,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CallType {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CreationMethod {
    Create,
    Create2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RewardType {
    Block,
    Uncle,
    EmptyStep,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Res {
    CallResult {
        gas_used: U256,
        output: Bytes,
    },
    CreateResult {
        gas_used: U256,
        code: Bytes,
        address: Address,
    },
    FailedCall(TraceError),
    FailedCreate(TraceError),
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TraceError {
    OutOfGas,
    BadJumpDestination,
    BadInstruction,
    StackUnderflow,
    OutOfStack,
    BuiltIn,
    Internal,
    MutableCallInStaticContext,
    Wasm,
    OutOfBounds,
    Reverted,
}
