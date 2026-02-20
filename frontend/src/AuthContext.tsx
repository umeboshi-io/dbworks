import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import type { AppUser } from './types';
import { api, setAuthToken } from './api/client';

interface AuthContextType {
  user: AppUser | null;
  isLoading: boolean;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType>({
  user: null,
  isLoading: true,
  logout: () => {},
});

export function useAuth() {
  return useContext(AuthContext);
}

const TOKEN_KEY = 'qo_token';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AppUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const loadUser = useCallback(async (token: string) => {
    setAuthToken(token);
    try {
      const me = await api.getMe();
      setUser(me);
    } catch {
      // Invalid token
      localStorage.removeItem(TOKEN_KEY);
      setAuthToken(null);
      setUser(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    // Check for token in URL (OAuth callback redirect)
    const params = new URLSearchParams(window.location.search);
    const urlToken = params.get('token');

    if (urlToken) {
      localStorage.setItem(TOKEN_KEY, urlToken);
      // Clean the URL
      window.history.replaceState({}, '', window.location.pathname);
      loadUser(urlToken);
      return;
    }

    // Check for token in localStorage
    const storedToken = localStorage.getItem(TOKEN_KEY);
    if (storedToken) {
      loadUser(storedToken);
      return;
    }

    setIsLoading(false);
  }, [loadUser]);

  const logout = useCallback(() => {
    localStorage.removeItem(TOKEN_KEY);
    setAuthToken(null);
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider value={{ user, isLoading, logout }}>
      {children}
    </AuthContext.Provider>
  );
}
