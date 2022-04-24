import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { EventsummariesService } from './eventsummaries.service';
import { Event_SummariesCreateInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-create.input';
import { Event_SummariesUpdateInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-update.input';
import { Event_SummariesWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-where-unique.input';


@Resolver('Eventsummary')
export class EventsummariesResolver {
  constructor(private eventsummariesService: EventsummariesService) {}

  @Mutation('createEventsummary')
  async create(
    @Args('createEventsummaryInput') createEventsummaryInput: Event_SummariesCreateInput
  ) {
    const created = await this.eventsummariesService.create(createEventsummaryInput);
  }

  @Query('eventsummaries')
  findAll() {
    return this.eventsummariesService.findAll();
  }

  @Query('eventsummary')
  findOne(@Args('monitorid') MonitorId: number) {
    return this.eventsummariesService.findOne({ MonitorId });
  }

  @Mutation('updateEventsummary')
  update(
    @Args('monitorid') MonitorId: number,
    @Args('updateEventsummaryInput') updateEventsummaryInput: Event_SummariesUpdateInput) {
    return this.eventsummariesService.update({MonitorId}, updateEventsummaryInput);
  }

  @Mutation('removeEventsummary')
  remove(@Args('monitorid') MonitorId: number) {
    return this.eventsummariesService.remove( {MonitorId} );
  }
}
