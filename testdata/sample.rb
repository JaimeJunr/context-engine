class UserSession
  belongs_to :user
  has_many :tokens

  def initialize(user)
    @user = user
  end

  def self.find_by_token(token)
    # ...
  end

  def valid?
    @user.active?
  end
end

module Auth
  class SessionManager
    def create(params)
    end
  end
end
