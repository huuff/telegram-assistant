{ telegram-assistant }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.telegram-assistant;
in
{
  options.services.telegram-assistant = {
    enable = lib.mkEnableOption "Telegram bot for interacting with LLMs";

    logLevel = lib.mkOption {
      type = lib.types.enum [
        "error"
        "warn"
        "info"
        "debug"
        "trace"
      ];
      default = "info";
      description = "Default log level for the service";
    };

    allowedUsers = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Whitelist of telegram user IDs that will be allowed to use the bot";
    };

    # MAYBE: should maybe be telegramTokenPath?
    telegramToken = lib.mkOption {
      type = lib.types.path;
      description = "Path to a file containing the Telegram token";
    };

    # MAYBE: should maybe be openrouterTokenPath?
    openrouterToken = lib.mkOption {
      type = lib.types.path;
      description = "Path to a file containing the OpenRouter token";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.telegram-assistant = {
      description = "Telegram bot for interacting with LLMs";

      wantedBy = [ "multi-user.target" ];

      environment = {
        RUST_LOG = cfg.logLevel;
        ALLOWED_USERS = lib.mkIf (cfg.allowedUsers != [ ]) (lib.concatStringsSep "," cfg.allowedUsers);
      };

      serviceConfig = {
        Restart = "always";

        ExecStart = ''
          ${pkgs.bash}/bin/bash -c 'TELEGRAM_TOKEN="$(cat $CREDENTIALS_DIRECTORY/telegram_token)" OPENROUTER_TOKEN="$(cat $CREDENTIALS_DIRECTORY/openrouter_token)" ${telegram-assistant}/bin/telegram-assistant'
        '';
        LoadCredential = [
          "telegram_token:${cfg.telegramToken}" # TODO: this env var should be renamed to TELEGRAM_TOKEN right?
          "openrouter_token:${cfg.openrouterToken}"
        ];
      };
    };
  };
}
