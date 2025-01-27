use clap::{arg, command, Parser};
use soroban_env_host::{
    xdr::{
        Asset, ContractDataDurability, ContractExecutable, ContractIdPreimage, CreateContractArgs,
        Error as XdrError, Hash, HostFunction, InvokeHostFunctionOp, LedgerKey::ContractData,
        LedgerKeyContractData, Memo, MuxedAccount, Operation, OperationBody, Preconditions,
        ScAddress, ScVal, SequenceNumber, Transaction, TransactionExt, Uint256, VecM,
    },
    HostError,
};
use std::convert::Infallible;
use std::{array::TryFromSliceError, fmt::Debug, num::ParseIntError};

use crate::{
    commands::{config, global, NetworkRunnable},
    rpc::{Client, Error as SorobanRpcError},
    utils::{contract_id_hash_from_asset, parsing::parse_asset},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    // TODO: the Display impl of host errors is pretty user-unfriendly
    //       (it just calls Debug). I think we can do better than that
    Host(#[from] HostError),
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Client(#[from] SorobanRpcError),
    #[error("internal conversion error: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("xdr processing error: {0}")]
    Xdr(#[from] XdrError),
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error(transparent)]
    ParseAssetError(#[from] crate::utils::parsing::Error),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    /// ID of the Hcnet classic asset to wrap, e.g. "USDC:G...5"
    #[arg(long)]
    pub asset: String,

    #[command(flatten)]
    pub config: config::Args,
    #[command(flatten)]
    pub fee: crate::fee::Args,
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let res_str = self.run_against_rpc_server(None, None).await?;
        println!("{res_str}");
        Ok(())
    }
}

#[async_trait::async_trait]
impl NetworkRunnable for Cmd {
    type Error = Error;
    type Result = String;

    async fn run_against_rpc_server(
        &self,
        _: Option<&global::Args>,
        config: Option<&config::Args>,
    ) -> Result<String, Error> {
        let config = config.unwrap_or(&self.config);
        // Parse asset
        let asset = parse_asset(&self.asset)?;

        let network = config.get_network()?;
        let client = Client::new(&network.rpc_url)?;
        client
            .verify_network_passphrase(Some(&network.network_passphrase))
            .await?;
        let key = config.key_pair()?;

        // Get the account sequence number
        let public_strkey =
            hcnet_strkey::ed25519::PublicKey(key.verifying_key().to_bytes()).to_string();
        // TODO: use symbols for the method names (both here and in serve)
        let account_details = client.get_account(&public_strkey).await?;
        let sequence: i64 = account_details.seq_num.into();
        let network_passphrase = &network.network_passphrase;
        let contract_id = contract_id_hash_from_asset(&asset, network_passphrase)?;
        let tx = build_wrap_token_tx(
            &asset,
            &contract_id,
            sequence + 1,
            self.fee.fee,
            network_passphrase,
            &key,
        )?;
        let txn = client.create_assembled_transaction(&tx).await?;
        let txn = self.fee.apply_to_assembled_txn(txn);
        client
            .send_assembled_transaction(txn, &key, &[], network_passphrase, None, None)
            .await?;

        Ok(hcnet_strkey::Contract(contract_id.0).to_string())
    }
}

fn build_wrap_token_tx(
    asset: &Asset,
    contract_id: &Hash,
    sequence: i64,
    fee: u32,
    _network_passphrase: &str,
    key: &ed25519_dalek::SigningKey,
) -> Result<Transaction, Error> {
    let contract = ScAddress::Contract(contract_id.clone());
    let mut read_write = vec![
        ContractData(LedgerKeyContractData {
            contract: contract.clone(),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
        }),
        ContractData(LedgerKeyContractData {
            contract: contract.clone(),
            key: ScVal::Vec(Some(
                vec![ScVal::Symbol("Metadata".try_into().unwrap())].try_into()?,
            )),
            durability: ContractDataDurability::Persistent,
        }),
    ];
    if asset != &Asset::Native {
        read_write.push(ContractData(LedgerKeyContractData {
            contract,
            key: ScVal::Vec(Some(
                vec![ScVal::Symbol("Admin".try_into().unwrap())].try_into()?,
            )),
            durability: ContractDataDurability::Persistent,
        }));
    }

    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::CreateContract(CreateContractArgs {
                contract_id_preimage: ContractIdPreimage::Asset(asset.clone()),
                executable: ContractExecutable::HcnetAsset,
            }),
            auth: VecM::default(),
        }),
    };

    Ok(Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(key.verifying_key().to_bytes())),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![op].try_into()?,
        ext: TransactionExt::V0,
    })
}
