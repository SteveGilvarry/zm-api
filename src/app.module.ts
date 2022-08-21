import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { ApolloDriver, ApolloDriverConfig } from '@nestjs/apollo';
import { GraphQLModule } from '@nestjs/graphql';
import { ApolloServerPluginLandingPageLocalDefault } from 'apollo-server-core';
import { MonitorsModule } from './monitors/monitors.module';
import { GraphQLDateTime, GraphQLBigInt } from 'graphql-scalars';
import { UsersModule } from './users/users.module';
import { ConfigModule } from './config/config.module';
import { ControlpresetsModule } from './controlpresets/controlpresets.module';
import { ControlsModule } from './controls/controls.module';
import { DevicesModule } from './devices/devices.module';
import { EventsummariesModule } from './eventsummaries/eventsummaries.module';
import { EventsModule } from './events/events.module';
import { FramesModule } from './frames/frames.module';
import { GroupsModule } from './groups/groups.module';
import { LogsModule } from './logs/logs.module';
import { ManufacturersModule } from './manufacturers/manufacturers.module';
import { ModelsModule } from './models/models.module';
import { MonitorstatusModule } from './monitorstatus/monitorstatus.module';
import { ServersModule } from './servers/servers.module';
import { StatesModule } from './states/states.module';
import { StorageModule } from './storage/storage.module';
import { ZonesModule } from './zones/zones.module';
import { ZonepresetsModule } from './zonepresets/zonepresets.module';
import { MonitorpresetsModule } from './monitorpresets/monitorpresets.module';
import { FiltersModule } from './filters/filters.module';
import { MontagelayoutsModule } from './montagelayouts/montagelayouts.module';

@Module({
  imports: [
    GraphQLModule.forRoot<ApolloDriverConfig>({
      driver: ApolloDriver,
      playground: false,
      plugins: [ApolloServerPluginLandingPageLocalDefault()],
      typePaths: ['./**/*.graphql'],
      resolvers: { DateTime: GraphQLDateTime, BigInt: GraphQLBigInt },
    }),
    MonitorsModule,
    UsersModule,
    ConfigModule,
    ControlpresetsModule,
    ControlsModule,
    DevicesModule,
    EventsummariesModule,
    EventsModule,
    FramesModule,
    GroupsModule,
    LogsModule,
    ManufacturersModule,
    ModelsModule,
    MonitorstatusModule,
    ServersModule,
    StatesModule,
    StorageModule,
    ZonesModule,
    ZonepresetsModule,
    MonitorpresetsModule,
    FiltersModule,
    MontagelayoutsModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
