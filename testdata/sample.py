class LoginService:
    def authenticate(self, username, password):
        pass

    def logout(self, session_id):
        pass

class TokenManager:
    def generate(self, user):
        pass

    def validate(self, token):
        pass
