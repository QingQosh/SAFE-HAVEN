/**
 * Network configuration utilities and constants
 * Supports Stellar testnet and mainnet only
 */

export enum NetworkId {
  TESTNET = 'testnet',
  MAINNET = 'mainnet',
}

export interface NetworkConfig {
  id: NetworkId
  name: string
  displayName: string
  color: 'red' | 'green'
  passphrase: string
  rpcUrl: string
  horizonUrl: string
  explorerUrl: string
  nativeToken: string
}

/** Testnet configuration */
export const TESTNET: NetworkConfig = {
  id: NetworkId.TESTNET,
  name: 'testnet',
  displayName: 'Testnet',
  color: 'red',
  passphrase: 'Test SDF Network ; September 2015',
  rpcUrl: 'https://soroban-testnet.stellar.org',
  horizonUrl: 'https://horizon-testnet.stellar.org',
  explorerUrl: 'https://stellar.expert/explorer/testnet',
  nativeToken: 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC',
}

/** Mainnet configuration */
export const MAINNET: NetworkConfig = {
  id: NetworkId.MAINNET,
  name: 'mainnet',
  displayName: 'Mainnet',
  color: 'green',
  passphrase: 'Public Global Stellar Network ; September 2015',
  rpcUrl: 'https://soroban.stellar.org',
  horizonUrl: 'https://horizon.stellar.org',
  explorerUrl: 'https://stellar.expert/explorer/public',
  nativeToken: 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC',
}

/** Map of network IDs to configurations */
export const NETWORKS: Record<NetworkId, NetworkConfig> = {
  [NetworkId.TESTNET]: TESTNET,
  [NetworkId.MAINNET]: MAINNET,
}

/** All available networks as array */
export const NETWORK_LIST: NetworkConfig[] = [TESTNET, MAINNET]

/**
 * Get network configuration by ID
 * @param networkId Network ID (testnet or mainnet)
 * @returns Network configuration
 */
export function getNetworkConfig(networkId: NetworkId): NetworkConfig {
  return NETWORKS[networkId]
}

/**
 * Get network ID from passphrase
 * @param passphrase Stellar network passphrase
 * @returns Network ID or undefined if not recognized
 */
export function getNetworkIdByPassphrase(
  passphrase: string
): NetworkId | undefined {
  if (passphrase === TESTNET.passphrase) return NetworkId.TESTNET
  if (passphrase === MAINNET.passphrase) return NetworkId.MAINNET
  return undefined
}

/**
 * Get network configuration from passphrase
 * @param passphrase Stellar network passphrase
 * @returns Network configuration or undefined if not recognized
 */
export function getNetworkByPassphrase(
  passphrase: string
): NetworkConfig | undefined {
  const networkId = getNetworkIdByPassphrase(passphrase)
  return networkId ? getNetworkConfig(networkId) : undefined
}

/**
 * Detect network from environment variables
 * @returns Detected network ID based on VITE_NETWORK_PASSPHRASE
 */
export function detectNetworkFromEnv(): NetworkId {
  const passphrase = import.meta.env.VITE_NETWORK_PASSPHRASE as string
  const networkId = getNetworkIdByPassphrase(passphrase)
  
  if (!networkId) {
    console.warn(
      `Unknown network passphrase: ${passphrase}. Defaulting to testnet.`
    )
    return NetworkId.TESTNET
  }
  
  return networkId
}

/**
 * Determine the opposite network
 * @param networkId Current network ID
 * @returns The other network's ID
 */
export function getAlternateNetwork(networkId: NetworkId): NetworkId {
  return networkId === NetworkId.TESTNET ? NetworkId.MAINNET : NetworkId.TESTNET
}

/**
 * Get badge color class for network
 * @param networkId Network ID
 * @returns Tailwind color class (bg-red-* or bg-green-*)
 */
export function getNetworkBadgeColor(networkId: NetworkId): string {
  const network = getNetworkConfig(networkId)
  return network.color === 'red' ? 'bg-red-600' : 'bg-green-600'
}

/**
 * Get badge text color class for network
 * @param networkId Network ID
 * @returns Tailwind color class for text
 */
export function getNetworkBadgeTextColor(networkId: NetworkId): string {
  const network = getNetworkConfig(networkId)
  return network.color === 'red' ? 'text-red-100' : 'text-green-100'
}
