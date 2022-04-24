import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsWhereInput } from './stats-where.input';

@ArgsType()
export class DeleteManyStatsArgs {

    @Field(() => StatsWhereInput, {nullable:true})
    where?: StatsWhereInput;
}
