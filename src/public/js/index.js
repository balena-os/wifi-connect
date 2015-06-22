$(function(){
	$.get("/ssids", function(data){
		$.each(data, function(i, val){
			$("#ssid-select").append("<option value='" + val + "'>" + val + "</option>");
		});		
	})
});