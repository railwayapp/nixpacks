defmodule ElixirNoEcto.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    IO.puts("Hello from Phoenix")
    children = [
      # Start the Telemetry supervisor
      ElixirNoEctoWeb.Telemetry,
      # Start the PubSub system
      {Phoenix.PubSub, name: ElixirNoEcto.PubSub},
      # Start Finch
      {Finch, name: ElixirNoEcto.Finch},
      # Start the Endpoint (http/https)
      ElixirNoEctoWeb.Endpoint
      # Start a worker by calling: ElixirNoEcto.Worker.start_link(arg)
      # {ElixirNoEcto.Worker, arg}
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: ElixirNoEcto.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    ElixirNoEctoWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
