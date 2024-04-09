import { ContractSpec, Address } from '@hcnet/hcnet-sdk';
import { Buffer } from "buffer";
import {
  AssembledTransaction,
  ContractClient,
  ContractClientOptions,
} from '@hcnet/hcnet-sdk/lib/contract_client/index.js';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@hcnet/hcnet-sdk/lib/contract_client';
import { Result } from '@hcnet/hcnet-sdk/lib/rust_types/index.js';
export * from '@hcnet/hcnet-sdk'
export * from '@hcnet/hcnet-sdk/lib/contract_client/index.js'
export * from '@hcnet/hcnet-sdk/lib/rust_types/index.js'

if (typeof window !== 'undefined') {
    //@ts-ignore Buffer exists
    window.Buffer = window.Buffer || Buffer;
}
