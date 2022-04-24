import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsUpdateInput } from './monitors-update.input';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';

@ArgsType()
export class UpdateOneMonitorsArgs {

    @Field(() => MonitorsUpdateInput, {nullable:false})
    data!: MonitorsUpdateInput;

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    where!: MonitorsWhereUniqueInput;
}
