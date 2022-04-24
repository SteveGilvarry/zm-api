import { Injectable } from '@nestjs/common';
import { Event_SummariesCreateInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-create.input';
import { Event_SummariesUpdateInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-update.input';
import { Event_SummariesWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/event-summaries/event-summaries-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class EventsummariesService {
  constructor(private prisma: PrismaService) {}

  create(createEventsummaryInput: Event_SummariesCreateInput) {
    return this.prisma.event_Summaries.create({
      data: createEventsummaryInput,
    });
  }

  findAll() {
    return this.prisma.event_Summaries.findMany();
  }

  findOne(event_SummariesWhereUniqueInput: Event_SummariesWhereUniqueInput) {
    return this.prisma.event_Summaries.findUnique({
      where: event_SummariesWhereUniqueInput,
    });

  }

  update(
    event_SummariesWhereUniqueInput: Event_SummariesWhereUniqueInput,
    updateEventsummaryInput: Event_SummariesUpdateInput,
  ) {
    return this.prisma.event_Summaries.update({
      where: event_SummariesWhereUniqueInput,
      data: updateEventsummaryInput,
    });
  }

  remove(event_SummariesWhereUniqueInput: Event_SummariesWhereUniqueInput) {
    return this.prisma.event_Summaries.delete({
      where: event_SummariesWhereUniqueInput,
    });
  }
}
