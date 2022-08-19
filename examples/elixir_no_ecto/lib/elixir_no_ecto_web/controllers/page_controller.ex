defmodule ElixirNoEctoWeb.PageController do
  use ElixirNoEctoWeb, :controller

  def index(conn, _params) do
    render(conn, "index.html")
  end
end
