# rs-datasvc
Restful frontend to Datahub
  
GET /tags                   => all tags (paged)  
GET /tags?query=<blah>      => tags with names like <blah> up to limit (default:10)  
GET /datasets               => all datasets (paged)  
GET /datasets?query=<blah>  => datasets with names like <blah> up to limit (default:10)  
GET /datasets?tags=<blah>   => datasets with tags like <blah> (paged)  
  
paged routes support: offset & limit query parameters  
not-paged routes support: limit query parameter  
