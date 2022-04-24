import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatsUpdateInput } from './stats-update.input';
import { StatsWhereUniqueInput } from './stats-where-unique.input';

@ArgsType()
export class UpdateOneStatsArgs {

    @Field(() => StatsUpdateInput, {nullable:false})
    data!: StatsUpdateInput;

    @Field(() => StatsWhereUniqueInput, {nullable:false})
    where!: StatsWhereUniqueInput;
}
