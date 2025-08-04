export interface AuthStatus {
  is_authenticated: boolean;
  user_id?: string;
  expires_at?: string;
  permissions: string[];
}

export type VersionStatus = 
  | "Current"
  | { UpdateRequired: string }
  | { UpdateAvailable: string };

export interface AuthRequest {
  user_key: string;
  client_version: string;
}

export interface AuthResponse {
  auth_status: AuthStatus;
  version_status: VersionStatus;
  server_message?: string;
}