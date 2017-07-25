Pony Playground
---------------

Useful commands, these require sudo first:

Restarting the playground
# systemctl restart playpen.service

Updating the Docker image (on a new ponyc release)
# systemctl start playpen-update.service

Updating the playground
# cd /opt/pony-playpen
# git pull
# cargo build --release
# systemctl restart playpen.service

Showing logs
# journalctl -ru playpen.service
# journalctl -ru playpen-update.service


---------------
