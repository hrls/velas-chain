use evm_rpc::traces::{
    BlockNumber, RpcResult, Trace, TraceOptions, TraceResultsWithTransactionHash, TracesERPC,
};

use crate::rpc::JsonRpcRequestProcessor;

pub struct TracesErpcImpl;

impl TracesERPC for TracesErpcImpl {
    type Metadata = JsonRpcRequestProcessor;

    fn block_traces(&self, block: BlockNumber) -> RpcResult<Option<Vec<Trace>>> {
        todo!()
    }

    fn replay_block_transactions(
        &self,
        block: BlockNumber,
        options: TraceOptions,
    ) -> RpcResult<Vec<TraceResultsWithTransactionHash>> {
        todo!()
    }
}
