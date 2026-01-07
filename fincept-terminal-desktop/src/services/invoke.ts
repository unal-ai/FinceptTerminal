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
  
  // Handle RPC response format - check for error first
  if (result.error) {
    throw new Error(result.error);
  }
  
  if (result.success === false) {
    throw new Error(result.error || 'Unknown error');
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
  
  getPeriodReturns: (symbol: string) =>
    invoke<PeriodReturnsResponse>('get_period_returns', { symbol }),
  
  checkMarketDataHealth: () =>
    invoke<boolean>('check_market_data_health', {}),
  
  // Database Commands
  dbCheckHealth: () =>
    invoke<HealthCheckResponse>('db_check_health', {}),
  
  dbGetAllSettings: () =>
    invoke<Setting[]>('db_get_all_settings', {}),
  
  dbGetSetting: (key: string) =>
    invoke<string | null>('db_get_setting', { key }),
  
  dbSaveSetting: (key: string, value: string, category?: string) =>
    invoke<{ saved: boolean }>('db_save_setting', { key, value, category }),
  
  // Watchlist Commands
  dbGetWatchlists: () =>
    invoke<Watchlist[]>('db_get_watchlists', {}),
  
  dbCreateWatchlist: (name: string, description?: string, color?: string) =>
    invoke<Watchlist>('db_create_watchlist', { name, description, color }),
  
  dbGetWatchlistStocks: (watchlistId: string) =>
    invoke<WatchlistStock[]>('db_get_watchlist_stocks', { watchlistId }),
  
  dbAddWatchlistStock: (watchlistId: string, symbol: string, notes?: string) =>
    invoke<WatchlistStock>('db_add_watchlist_stock', { watchlistId, symbol, notes }),
  
  dbRemoveWatchlistStock: (watchlistId: string, symbol: string) =>
    invoke<{ removed: boolean }>('db_remove_watchlist_stock', { watchlistId, symbol }),
  
  dbDeleteWatchlist: (watchlistId: string) =>
    invoke<{ deleted: boolean }>('db_delete_watchlist', { watchlistId }),
  
  // Credential Commands
  dbGetCredentials: () =>
    invoke<Credential[]>('db_get_credentials', {}),
  
  dbSaveCredential: (credential: Credential) =>
    invoke<{ success: boolean; message: string }>('db_save_credential', credential),
  
  dbGetCredentialByService: (serviceName: string) =>
    invoke<Credential | null>('db_get_credential_by_service', { serviceName }),
  
  dbDeleteCredential: (id: number) =>
    invoke<{ success: boolean; message: string }>('db_delete_credential', { id }),
  
  // LLM Config Commands
  dbGetLlmConfigs: () =>
    invoke<LlmConfig[]>('db_get_llm_configs', {}),
  
  dbSaveLlmConfig: (config: LlmConfig) =>
    invoke<{ saved: boolean }>('db_save_llm_config', config),
  
  dbGetLlmGlobalSettings: () =>
    invoke<LlmGlobalSettings>('db_get_llm_global_settings', {}),
  
  dbSaveLlmGlobalSettings: (settings: LlmGlobalSettings) =>
    invoke<{ saved: boolean }>('db_save_llm_global_settings', settings),
  
  // Chat Session Commands
  dbCreateChatSession: (title: string) =>
    invoke<ChatSession>('db_create_chat_session', { title }),
  
  dbGetChatSessions: (limit?: number) =>
    invoke<ChatSession[]>('db_get_chat_sessions', { limit }),
  
  dbAddChatMessage: (message: ChatMessage) =>
    invoke<ChatMessage>('db_add_chat_message', message),
  
  dbGetChatMessages: (sessionUuid: string) =>
    invoke<ChatMessage[]>('db_get_chat_messages', { sessionUuid }),
  
  dbDeleteChatSession: (sessionUuid: string) =>
    invoke<{ deleted: boolean }>('db_delete_chat_session', { sessionUuid }),
  
  // Data Source Commands
  dbGetAllDataSources: () =>
    invoke<DataSource[]>('db_get_all_data_sources', {}),
  
  dbSaveDataSource: (source: DataSource) =>
    invoke<{ success: boolean; message: string; id?: string }>('db_save_data_source', source),
  
  dbDeleteDataSource: (id: string) =>
    invoke<{ success: boolean; message: string }>('db_delete_data_source', { id }),
  
  // Portfolio Commands
  dbListPortfolios: () =>
    invoke<Portfolio[]>('db_list_portfolios', {}),
  
  dbGetPortfolio: (portfolioId: string) =>
    invoke<Portfolio | null>('db_get_portfolio', { portfolioId }),
  
  dbCreatePortfolio: (name: string, currency?: string, description?: string) =>
    invoke<{ id: string; created: boolean }>('db_create_portfolio', { name, currency, description }),
  
  dbDeletePortfolio: (portfolioId: string) =>
    invoke<{ deleted: boolean }>('db_delete_portfolio', { portfolioId }),
  
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

export interface PeriodReturnsResponse {
  symbol: string;
  seven_day: number;
  thirty_day: number;
}

export interface HealthCheckResponse {
  status: string;
  message: string;
}

export interface Setting {
  setting_key: string;
  setting_value: string;
  category?: string;
  updated_at: string;
}

export interface Watchlist {
  id: string;
  name: string;
  description?: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface WatchlistStock {
  id: string;
  watchlist_id: string;
  symbol: string;
  notes?: string;
  added_at: string;
}

export interface Credential {
  id?: number;
  service_name: string;
  username?: string;
  password?: string;
  api_key?: string;
  api_secret?: string;
  additional_data?: string;
  created_at?: string;
  updated_at?: string;
}

export interface LlmConfig {
  provider: string;
  api_key?: string;
  base_url?: string;
  model?: string;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface LlmGlobalSettings {
  temperature?: number;
  max_tokens?: number;
  system_prompt?: string;
}

export interface ChatSession {
  session_uuid: string;
  title: string;
  message_count: number;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  session_uuid: string;
  role: string;
  content: string;
  timestamp?: string;
  provider?: string;
  model?: string;
  tokens_used?: number;
}

export interface DataSource {
  id: string;
  alias: string;
  display_name: string;
  description?: string;
  ds_type: string;
  provider: string;
  category: string;
  config?: string;
  enabled: boolean;
  tags?: string;
  created_at?: string;
  updated_at?: string;
}

export interface Portfolio {
  id: string;
  name: string;
  owner: string;
  currency: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface SetupStatus {
  needs_setup: boolean;
  python_installed: boolean;
  database_ready: boolean;
}

// Export for backward compatibility
export default invoke;
