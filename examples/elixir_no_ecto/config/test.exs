import Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :elixir_no_ecto, ElixirNoEctoWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "7WY5lOzw4yW9ArYKIeeQ4WvSI5JYRfqD5Naqj0F4dDTjQFIQ0DXY4+oU43Hl9LES",
  server: false

# In test we don't send emails.
config :elixir_no_ecto, ElixirNoEcto.Mailer,
  adapter: Swoosh.Adapters.Test

# Print only warnings and errors during test
config :logger, level: :warn

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime
