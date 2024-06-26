<p align="center">
    <h3 align="center">Busser</h3>
    <p align="center">
        Simply host your corner of the internet in Rust
        <br>
    </p>
</p>

<div align = "center">
    
[![Linux x86_64](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-linux.yml/badge.svg)](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-linux.yml) [![macOS](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-macos.yml/badge.svg)](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-macos.yml) [![Windows](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-windows.yml/badge.svg)](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-windows.yml) 

[![aarch64](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-aarch64.yml/badge.svg)](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-aarch64.yml) [![armv7](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-armv7.yml/badge.svg)](https://github.com/JerboaBurrow/Busser/actions/workflows/build-and-test-armv7.yml)

[![Coverage Status](https://coveralls.io/repos/github/JerboaBurrow/Busser/badge.svg?branch=main)](https://coveralls.io/github/JerboaBurrow/Busser?branch=main)
</div>

✔️ Quickly host static sites, via free tier cloud services (Google Cloud e2-micro) or a Raspberry Pi!

✔️ Git based content management (with Github webhook integration) with automated checkouts, content re-serving, and sitemap generation.

✔️ Status messages on content updates and hit statistics (Currently via Discord webhook integration)

✔️ URL shortening, e.g. ```/x/y/z/webpage.html``` aliased as ```/x/y/z/webpage```

✔️ Http redirect to https and Https certificates

✔️ IP throttling, and anonymised hit statistics 

✔️ Hot :fire: loadable configuration

# Contents

- [Planned features](#planned-features)
- [Spinning up](#spinning-up)
    - [Configuration](#configuration)
- [Free static website hosting example with Google Cloud Free Tier](#free-static-website-hosting-example-with-google-cloud-free-tier)
___

# Planned features

- Zulip, Slack, etc. webhook integration.
- Gitlab webhook integration
- Proxy relaying (e.g. relay POSTS to AWS Lambda based apis)
- System health status messages.
- System alerts (user configurable burst events, RAM/DISC usage, etc.)

___

# Spinning up

1. Just create a folder with your ```.html/.css/.js``` and other resources, ```.png, .gif, ...```
2. Point Busser to it with a config.json
3. Run it, and that's it!*

\* you'll need certificates for https, and open ports

## Configuration

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
        "path": "stats",
        "hit_cooloff_seconds": 3600,
        "save_schedule": "0 0 * * * * *",
        "digest_schedule": "0 0 0 * * * *",
        "ignore_regexes": ["/favicon.ico"]
    },
    "content": 
    {
        "path": "PATH_TO_SITE_FILES",
        "home": "PATH_TO_SITE_ROOT_PAGE",
        "allow_without_extension": true,
        "browser_cache_period_seconds": 3600,
        "server_cache_period_seconds": 3600,
        "ignore_regexes": ["/.git", "workspace"],
        "generate_sitemap": true,
        "message_on_sitemap_reload": true
    },
    "git":
    {
        "remote": "git@github.com:JerboaBurrow/website.git",
        "branch": "main",
        "checkout_schedule": "10 * * * * * *",
        "remote_webhook_token": "GITHUB_WEBHOOK_SECRET",
        "auth":
        {
            "key_path": "YOUR_KEY_PATH",
            "user": "Jerboa-app",
            "passphrase": "YOUR_KEY_PASS"
        }
    },
    "domain": "127.0.0.1",
    "api_token": "YOUR_BUSSER_API_TOKEN",
    "notification_endpoint": { "addr": "https://discord.com/api/webhooks/xxx/yyy" },
    "cert_path": "certs/cert.pem",
    "key_path": "certs/key.pem"
}

```
____

## GDPR, Cookie Policies, and Privacy Policies

- The IP throttler only stores hashes of an IP and a request path, it is likely not considered identifiable information.

- The statistics collection stores the IP, hit time, path, and counts for each IP-path pair. The IP is stored as a hash value.
____

## API

Currently there is an API function to request a statistics digest, the following bash script will perform the request. It is currently limited to only returning stats based on already saved data.

```
# ./get_stats.sh the_secret_token '{"from_utc":"2024-03-07T08:40:50.948868839+00:00","post_discord": false}'
hmac=$(echo -n $2 | openssl dgst -sha256 -hmac $1 | sed 's/SHA2-256(stdin)= //g') 
curl -v -H 'Content-Type: application/json' -d "$2" -H 'api: StatsDigest' -H "busser-token: ${hmac}" -X POST https://your.domain
```
___

# Free static website hosting example with Google Cloud Free Tier

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
