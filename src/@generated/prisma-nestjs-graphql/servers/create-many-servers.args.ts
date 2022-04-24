import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersCreateManyInput } from './servers-create-many.input';

@ArgsType()
export class CreateManyServersArgs {

    @Field(() => [ServersCreateManyInput], {nullable:false})
    data!: Array<ServersCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
