import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsCreateInput } from './monitors-create.input';

@ArgsType()
export class CreateOneMonitorsArgs {

    @Field(() => MonitorsCreateInput, {nullable:false})
    data!: MonitorsCreateInput;
}
