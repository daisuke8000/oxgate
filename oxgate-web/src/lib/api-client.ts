const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: any
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function fetchApi<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const url = `${API_URL}${endpoint}`;

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
    });

    const data = await response.json();

    if (!response.ok) {
      throw new ApiError(
        response.status,
        data.error || "unknown_error",
        data.message || "An error occurred",
        data.details
      );
    }

    return data;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(0, "network_error", "Network error occurred");
  }
}

export const apiClient = {
  login: (data: {
    login_challenge: string;
    email: string;
    password: string;
  }) => fetchApi<{ redirect_to: string }>("/api/login", {
    method: "POST",
    body: JSON.stringify(data),
  }),

  consent: (data: {
    consent_challenge: string;
    accept: boolean;
    grant_scope?: string[];
  }) => fetchApi<{ redirect_to: string }>("/api/consent", {
    method: "POST",
    body: JSON.stringify(data),
  }),

  logout: (data: { logout_challenge: string }) =>
    fetchApi<{ redirect_to: string }>("/api/logout", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  register: (data: { email: string; password: string }) =>
    fetchApi<{ user_id: string }>("/api/register", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  requestPasswordReset: (data: { email: string }) =>
    fetchApi<{ message: string }>("/api/password-reset/request", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  confirmPasswordReset: (data: { token: string; new_password: string }) =>
    fetchApi<{ message: string }>("/api/password-reset/confirm", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  setup2FA: (data: { password: string }) =>
    fetchApi<{ secret: string; qr_code: string }>("/api/2fa/setup", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  verify2FA: (data: { code: string }) =>
    fetchApi<{ enabled: boolean }>("/api/2fa/verify", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  disable2FA: (data: { password: string; code: string }) =>
    fetchApi<{ disabled: boolean }>("/api/2fa/disable", {
      method: "POST",
      body: JSON.stringify(data),
    }),
};
