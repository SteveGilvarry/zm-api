import { Module } from '@nestjs/common';
import { EventsummariesService } from './eventsummaries.service';
import { EventsummariesResolver } from './eventsummaries.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,EventsummariesResolver, EventsummariesService]
})
export class EventsummariesModule {}
