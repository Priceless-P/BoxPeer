// Copyright © Aptos
// SPDX-License-Identifier: Apache-2.0

import { Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk";

export const LocalStorageKeys = {
  keylessAccounts: "@aptos-connect/keyless-accounts",
};

export const devnetClient = new Aptos(
  new AptosConfig({ network: Network.TESTNET })
);

/// FIXME: Put your client id here
export const GOOGLE_CLIENT_ID = "539990832699-0u5ctmkaecdvdl125gj0k4a177cj87b7.apps.googleusercontent.com";
