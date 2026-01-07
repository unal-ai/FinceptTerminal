/**
 * Unified Invoke Service
 * 
 * This service provides a unified API for calling backend commands that works
 * in both Tauri (desktop) and web environments.
 * 
 * - In Tauri: Uses native IPC via @tauri-apps/api/core invoke
 * - In Web: Uses HTTP fetch to /api/rpc endpoint
 * 
 * Usage:
 *   import { invoke } from '@/services/invoke';
 *   const result = await invoke('get_market_quote', { symbol: 'AAPL' });
 */

// Environment detection
export const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;
export const IS_WEB = !IS_TAURI;

// API base URL for web mode
const API_BASE = import.meta.env.VITE_API_URL || '/api';

/**
 * Unified invoke function that works in both Tauri and web environments
 * 
 * @param cmd - Command name (e.g., 'get_market_quote')
 * @param args - Command arguments as an object
 * @returns Promise resolving to the command result
 */
export async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (IS_TAURI) {
    // Use native Tauri IPC
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(cmd, args);
  } else {
    // Use HTTP RPC for web
    return invokeWeb<T>(cmd, args);
  }
}

/**
 * Web-mode invoke using HTTP fetch
 */
async function invokeWeb<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  const response = await fetch(`${API_BASE}/rpc`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ cmd, args }),
  });

  if (!response.ok) {
    throw new Error(`RPC call failed: ${response.status} ${response.statusText}`);
  }

  const result = await response.json();
  
  // Handle RPC response format
  if (result.success === false && result.error) {
    throw new Error(result.error);
  }

  // Return the data if it exists, otherwise return the whole result
  return result.data !== undefined ? result.data : result;
}

/**
 * Type-safe wrapper for commands with known return types
 */
export const commands = {
  // Market Data Commands
  getMarketQuote: (symbol: string) => 
    invoke<MarketQuoteResponse>('get_market_quote', { symbol }),
  
  getMarketQuotes: (symbols: string[]) => 
    invoke<MarketQuotesResponse>('get_market_quotes', { symbols }),
  
  getHistoricalData: (symbol: string, startDate: string, endDate: string) =>
    invoke<HistoricalResponse>('get_historical_data', { symbol, startDate, endDate }),
  
  getStockInfo: (symbol: string) =>
    invoke<StockInfoResponse>('get_stock_info', { symbol }),
  
  getFinancials: (symbol: string) =>
    invoke<FinancialsResponse>('get_financials', { symbol }),
  
  // Database Commands
  dbCheckHealth: () =>
    invoke<HealthCheckResponse>('db_check_health', {}),
  
  dbGetAllSettings: () =>
    invoke<Record<string, string>>('db_get_all_settings', {}),
  
  dbSaveSetting: (key: string, value: string) =>
    invoke<{ saved: boolean }>('db_save_setting', { key, value }),
  
  // Watchlist Commands
  dbGetWatchlists: () =>
    invoke<Watchlist[]>('db_get_watchlists', {}),
  
  dbCreateWatchlist: (name: string, description?: string) =>
    invoke<{ id: number }>('db_create_watchlist', { name, description }),
  
  dbGetWatchlistStocks: (watchlistId: number) =>
    invoke<WatchlistStock[]>('db_get_watchlist_stocks', { watchlistId }),
  
  dbAddWatchlistStock: (watchlistId: number, symbol: string, name?: string) =>
    invoke<{ added: boolean }>('db_add_watchlist_stock', { watchlistId, symbol, name }),
  
  dbRemoveWatchlistStock: (watchlistId: number, symbol: string) =>
    invoke<{ removed: boolean }>('db_remove_watchlist_stock', { watchlistId, symbol }),
  
  dbDeleteWatchlist: (id: number) =>
    invoke<{ deleted: boolean }>('db_delete_watchlist', { id }),
  
  // Credential Commands
  dbGetCredentials: () =>
    invoke<Credential[]>('db_get_credentials', {}),
  
  dbSaveCredential: (service: string, apiKey?: string, apiSecret?: string) =>
    invoke<{ saved: boolean }>('db_save_credential', { service, apiKey, apiSecret }),
  
  dbDeleteCredential: (service: string) =>
    invoke<{ deleted: boolean }>('db_delete_credential', { service }),
  
  // LLM Config Commands
  dbGetLlmConfigs: () =>
    invoke<LlmConfig[]>('db_get_llm_configs', {}),
  
  dbSaveLlmConfig: (config: LlmConfig) =>
    invoke<{ saved: boolean }>('db_save_llm_config', config),
  
  dbGetLlmGlobalSettings: () =>
    invoke<LlmGlobalSettings>('db_get_llm_global_settings', {}),
  
  dbSaveLlmGlobalSettings: (settings: LlmGlobalSettings) =>
    invoke<{ saved: boolean }>('db_save_llm_global_settings', settings),
  
  // Setup Commands
  checkSetupStatus: () =>
    invoke<SetupStatus>('check_setup_status', {}),
  
  // Utility Commands
  greet: (name: string) =>
    invoke<string>('greet', { name }),
  
  sha256Hash: (input: string) =>
    invoke<string>('sha256_hash', { input }),
};

// Type definitions for API responses
export interface MarketQuoteResponse {
  success: boolean;
  data?: {
    symbol: string;
    price: number;
    change: number;
    change_percent: number;
    volume?: number;
    high?: number;
    low?: number;
    open?: number;
    previous_close?: number;
    timestamp: number;
  };
  error?: string;
}

export interface MarketQuotesResponse {
  success: boolean;
  data: Array<{
    symbol: string;
    price: number;
    change: number;
    change_percent: number;
    volume?: number;
    timestamp: number;
  }>;
  error?: string;
}

export interface HistoricalResponse {
  success: boolean;
  data: Array<{
    symbol: string;
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    adj_close: number;
  }>;
  error?: string;
}

export interface StockInfoResponse {
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

export interface FinancialsResponse {
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

export interface HealthCheckResponse {
  status: string;
  message: string;
}

export interface Watchlist {
  id: number;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface WatchlistStock {
  id: number;
  watchlist_id: number;
  symbol: string;
  name?: string;
  added_at: string;
}

export interface Credential {
  id: number;
  service: string;
  api_key?: string;
  api_secret?: string;
  created_at: string;
  updated_at: string;
}

export interface LlmConfig {
  id?: number;
  name: string;
  provider: string;
  model: string;
  api_key?: string;
  base_url?: string;
  temperature?: number;
  max_tokens?: number;
  enabled: boolean;
}

export interface LlmGlobalSettings {
  default_provider?: string;
  default_model?: string;
  stream_responses?: boolean;
}

export interface SetupStatus {
  needs_setup: boolean;
  python_installed: boolean;
  database_ready: boolean;
}

// Export for backward compatibility
export default invoke;
