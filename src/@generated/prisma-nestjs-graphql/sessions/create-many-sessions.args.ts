import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsCreateManyInput } from './sessions-create-many.input';

@ArgsType()
export class CreateManySessionsArgs {

    @Field(() => [SessionsCreateManyInput], {nullable:false})
    data!: Array<SessionsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
