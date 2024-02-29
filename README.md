## Busser

#### Simple HTTPS website hosting in Rust with [Axum](https://github.com/tokio-rs/axum).

✔️ Host static HTML/css/js/text and image/video content (png, jpg, gif, webp, mp4, ...) content from a given directory

✔️ Serve with and without ```.html```, e.g. ```/x/y/z/webpage.html``` aliased as ```/x/y/z/webpage```

✔️ Http redirect to https

✔️ Https certificates

✔️ IP throttling

✔️ Hit statistics and statistics digest (Discord webhook integration)

✔️ Hot :fire: loadable configuration

✔️ Host via **free tier** cloud services!

🏗️ Discord webhook integration for status messages

🏗️ Authenticated API for status/statistics polling

##### Host for free on Google Cloud

1. Just create a folder with your ```.html/.css/.js``` and other resources, ```.png, .gif, ...```
2. Point Busser to it with a config.json
3. Run it, and that's it!*

\* you'll need certificates for https, and open ports

### Configuration

The ```config.json``` specifies key properties of the site and its content

```json
{
    "port_https": 443,
    "port_http": 80, 
    "throttle": 
    {
        "max_requests_per_second": 64.0, 
        "timeout_millis": 5000, 
        "clear_period_seconds": 3600
    },
    "stats": 
    {
        "save_period_seconds": 86400,
        "path": "stats",
        "hit_cooloff_seconds": 3600,
        "clear_period_seconds": 2419200,
        "digest_period_seconds": 604800,
        "log_files_clear_period_seconds": 2419200
    },
    "path": "/home/jerboa/Website/",
    "home": "/home/jerboa/Website/jerboa.html",
    "domain": "jerboa.app",
    "allow_without_extension": true,
    "notification_endpoint": { "addr": "https://discord.com/api/webhooks/xxx/yyy" },
    "cache_period_seconds": 3600,
    "cert_path": "certs/cert.pem",
    "key_path": "certs/key.pem"
}
```
____

#### GDPR, Cookie Policies, and Privacy Policies

- The IP throttler only stores hashes of an IP and a request path, it is likely not considered identifiable information.

- The statistics collection stores the IP, hit time, path, and counts for each IP-path pair. This may be considered identifieable information. Automatic deletion of this data is carried out, hot-configurable in config.json. 
____

### Free static website hosting example with Google Cloud Free Tier

The [gcloud free tier](https://cloud.google.com/free?hl=en) [allows for the following instance running 24/7:](https://cloud.google.com/free/docs/free-cloud-features#compute)

```
    1 non-preemptible e2-micro VM instance per month in one of the following US regions:
        Oregon: us-west1
        Iowa: us-central1
        South Carolina: us-east1
    30 GB-months standard persistent disk
    1 GB of outbound data transfer from North America to all region destinations (excluding China and Australia) per month

```

You will still see costs in the Google cloud console, or savings suggestions. You should recieve free tier discount deductions to nullify these cost completely

This can be verified by:

1. Navigating to the [Google Cloud Console](https://console.cloud.google.com)
2. Selecting ```Billing``` form the burger menu (top left as of now)
3. Selecting ```Cost Table```
4. In ```Filters``` (right) select ```SKUs```
5. Type ```e2```
6. Select the all (e.g. ```16 filtered results```)
7. Toggle the arrow for you project
8. Toggle ```Compute Engine```
9. You should see e.g. ```E2 Instance Core running in Americas``` with ```X.XX```
10. You should also see e.g.  ```E2 Instance Ram running with free tier discount``` with ```-X.XX```

As this is a website you may be charged for network costs over the 1 GB outbound data, or for traffic from China and Australia

##### Create it using the CLI...

Using the gloud cli this command should create an instance template for the free tier, which can be used to create instances

```bash
gcloud beta compute instance-templates create free-tier-template-http --project=YOUR_PROJECT --machine-type=e2-micro \\
--network-interface=network=default,network-tier=PREMIUM \\
--instance-template-region=projects/YOUR_PROJECT/regions/us-central1 --maintenance-policy=MIGRATE \\
--provisioning-model=STANDARD --service-account=YOUR_SERVICE_ACCOUNT \\
--scopes=https://www.googleapis.com/auth/devstorage.read_only,https://www.googleapis.com/auth/logging.write,https://www.googleapis.com/auth/monitoring.write,https://www.googleapis.com/auth/servicecontrol,https://www.googleapis.com/auth/service.management.readonly,https://www.googleapis.com/auth/trace.append \\
--enable-display-device --tags=http-server,https-server \\
--create-disk=auto-delete=yes,boot=yes,device-name=free-tier-template,image=projects/debian-cloud/global/images/debian-11-bullseye-v20220719,mode=rw,size=30,type=pd-standard 
--no-shielded-secure-boot --shielded-vtpm --shielded-integrity-monitoring --reservation-affinity=any
```

##### ...or using Cloud console

- create an e2 in us-central1 (Iowa) for both zone and region
- select e2-micro (0.25-2 vCPU 1GB memory)
- you can change the boot disc from 10GB to 30GB if you like
- allow HTTPS and HTTP (if you need it for certificate provising)
- all else as default

#### Network

Make sure 443 and 80 are open ports (or whatever ports you wish to serve on)

### https certificate setup

#### Self signed (useful for localhost testing)

- You can use the bash script ```certs/gen.sh``` to generate a key/cert pair with openssl

#### Production; from authority

- get a domain (e.g. from squarespace)
- create a custom DNS record, e.g.
    - ```your.domain.somewhere    A	1 hour	google.cloud.instance.ip ```
- Use [Let's Encrypts](https://letsencrypt.org/) recommendation of [certbot](https://certbot.eff.org/) it really is very easy
    - Something like ```sudo certbot certonly --standalone -d your.domain.somewhere -d sub.your.domain.somehwere```
    - You will need to enable http in the cloud instance firewall for provisioning as well as https

#### Spinning up

Either: Run at login, root may be required for certificates (should be for certbot ones)
  
Or: Use a service file in ```/lib/systemd/system```, e.g

```
[Unit]
Description=Busser

[Service]
ExecStart=busser -d
WorkingDirectory=/home/busser
User=root

[Install]
WantedBy=multi-user.target
```

Then start and monitor it with

```sudo systemctl start busser.service``` and ```sudo journalctl -e -u busser.service```
