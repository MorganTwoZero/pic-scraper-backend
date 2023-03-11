param
(
	[Parameter(Mandatory = $true, ValueFromPipeline = $true)]
	[string]	      
	$param1
)
ssh root@vpn "cd pic-scraper-backend && docker compose exec -u postgres db psql -d post -U postgres -c `"$param1`""