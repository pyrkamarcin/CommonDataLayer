FROM mcr.microsoft.com/dotnet/sdk:5.0
WORKDIR /App

COPY * ./
RUN dotnet restore

RUN dotnet build -o './publish'

WORKDIR /App/publish
ENTRYPOINT ["dotnet", "vstest", "CDL.Tests.dll", "/logger:console;verbosity=detailed"]