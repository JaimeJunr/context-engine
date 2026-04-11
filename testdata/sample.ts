interface AuthSession {
  userId: string;
  token: string;
}

class SessionController {
  create(params: AuthSession): void {}
  destroy(id: string): void {}
}

type LoginResult = {
  success: boolean;
  token?: string;
};
