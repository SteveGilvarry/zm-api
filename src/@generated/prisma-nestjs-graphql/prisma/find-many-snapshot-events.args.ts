import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Snapshot_EventsWhereInput } from '../snapshot-events/snapshot-events-where.input';
import { Type } from 'class-transformer';
import { Snapshot_EventsOrderByWithRelationInput } from '../snapshot-events/snapshot-events-order-by-with-relation.input';
import { Snapshot_EventsWhereUniqueInput } from '../snapshot-events/snapshot-events-where-unique.input';
import { Int } from '@nestjs/graphql';
import { Snapshot_EventsScalarFieldEnum } from '../snapshot-events/snapshot-events-scalar-field.enum';

@ArgsType()
export class FindManySnapshotEventsArgs {

    @Field(() => Snapshot_EventsWhereInput, {nullable:true})
    @Type(() => Snapshot_EventsWhereInput)
    where?: Snapshot_EventsWhereInput;

    @Field(() => [Snapshot_EventsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<Snapshot_EventsOrderByWithRelationInput>;

    @Field(() => Snapshot_EventsWhereUniqueInput, {nullable:true})
    cursor?: Snapshot_EventsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [Snapshot_EventsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof Snapshot_EventsScalarFieldEnum>;
}
