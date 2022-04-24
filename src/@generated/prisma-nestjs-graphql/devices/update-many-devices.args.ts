import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesUpdateManyMutationInput } from './devices-update-many-mutation.input';
import { DevicesWhereInput } from './devices-where.input';

@ArgsType()
export class UpdateManyDevicesArgs {

    @Field(() => DevicesUpdateManyMutationInput, {nullable:false})
    data!: DevicesUpdateManyMutationInput;

    @Field(() => DevicesWhereInput, {nullable:true})
    where?: DevicesWhereInput;
}
